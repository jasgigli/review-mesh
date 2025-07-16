use clap::{Parser, Subcommand};
use tui::{Terminal, backend::CrosstermBackend, widgets::{Block, Borders, Paragraph, List, ListItem, Wrap}, layout::{Layout, Constraint, Direction, Rect}, style::{Style, Color}};
use crossterm::{event::{self, KeyCode, KeyEvent, Event}, execute, terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use std::io;
use std::time::Duration;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use serde_json;
use common::{ChatLine, Comment, ReviewSession};
use network::Network;
use chrono::Utc;
use git_integration::compute_diff;
use storage::Storage;
use std::fs::File;
use std::io::Write;
use genpdf::{elements, Alignment, Document, style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    Command,
    Chat,
}

enum NetMsg {
    Chat(ChatLine),
    Comment(Comment),
}

struct App {
    input: String,
    input_mode: InputMode,
    chat: Vec<String>,
    comments: Vec<String>,
    status: String,
    session: ReviewSession,
    author: String,
    net_tx: Option<Sender<NetMsg>>,
    // Diff navigation state
    files: Vec<String>,
    hunks: Vec<common::DiffHunk>,
    selected_file: usize,
    selected_hunk: usize,
    storage: Option<Storage>,
}

impl App {
    fn new(session: ReviewSession, author: String, storage: Option<Storage>) -> Self {
        // Load diffs from git
        let hunks = compute_diff(".", "HEAD");
        let mut files = hunks.iter().map(|h| h.file.clone()).collect::<Vec<_>>();
        files.sort();
        files.dedup();
        // Load chat/comments from storage
        let mut chat = vec!["Welcome to ReviewMesh! Type /help for commands.".to_string()];
        let mut comments = vec![];
        if let Some(store) = &storage {
            if let Ok(loaded) = store.load_chat(&session.id) {
                for c in loaded {
                    chat.push(format!("{}: {}", c.author, c.body));
                }
            }
            if let Ok(loaded) = store.load_comments(&session.id) {
                for c in loaded {
                    comments.push(format!("{}: {}", c.author, c.body));
                }
            }
        }
        Self {
            input: String::new(),
            input_mode: InputMode::Chat,
            chat,
            comments,
            status: "Chat mode (Enter to send, / to enter command)".to_string(),
            session,
            author,
            net_tx: None,
            files,
            hunks,
            selected_file: 0,
            selected_hunk: 0,
            storage,
        }
    }

    fn current_file(&self) -> Option<&String> {
        self.files.get(self.selected_file)
    }

    fn current_hunks(&self) -> Vec<&common::DiffHunk> {
        if let Some(file) = self.current_file() {
            self.hunks.iter().filter(|h| &h.file == file).collect()
        } else {
            vec![]
        }
    }

    fn current_hunk(&self) -> Option<&common::DiffHunk> {
        let hunks = self.current_hunks();
        hunks.get(self.selected_hunk).copied()
    }

    fn export_markdown(&self) -> std::io::Result<()> {
        let mut file = File::create("reviewmesh_export.md")?;
        writeln!(file, "# ReviewMesh Export: {}", self.session.title)?;
        writeln!(file, "Session ID: {}", self.session.id)?;
        writeln!(file, "Created: {}", self.session.created_at)?;
        writeln!(file, "Participants: {:?}\n", self.session.participants)?;
        writeln!(file, "## Diffs\n")?;
        for file_name in &self.files {
            writeln!(file, "### File: {}\n", file_name)?;
            for hunk in self.hunks.iter().filter(|h| &h.file == file_name) {
                writeln!(file, "```diff\n{}\n```", hunk.content.trim_end())?;
                // Comments for this hunk
                for comment in self.comments.iter().filter(|c| c.contains(&hunk.id)) {
                    writeln!(file, "> {}", comment)?;
                }
            }
        }
        writeln!(file, "\n## Comments\n")?;
        for comment in &self.comments {
            writeln!(file, "- {}", comment)?;
        }
        writeln!(file, "\n## Chat\n")?;
        for chat in &self.chat {
            writeln!(file, "- {}", chat)?;
        }
        Ok(())
    }

    fn export_pdf(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut doc = Document::new(genpdf::fonts::from_files("/usr/share/fonts", "LiberationSans", None)?);
        doc.set_title(format!("ReviewMesh Export: {}", self.session.title));
        let mut decorator = style::Style::new();
        decorator.set_bold(true);
        doc.push(elements::Paragraph::new(format!("ReviewMesh Export: {}", self.session.title)).styled(decorator));
        doc.push(elements::Paragraph::new(format!("Session ID: {}", self.session.id)));
        doc.push(elements::Paragraph::new(format!("Created: {}", self.session.created_at)));
        doc.push(elements::Paragraph::new(format!("Participants: {:?}", self.session.participants)));
        doc.push(elements::Break::new(1));
        doc.push(elements::Paragraph::new("Diffs:").aligned(Alignment::Left));
        for file_name in &self.files {
            doc.push(elements::Paragraph::new(format!("File: {}", file_name)).styled(decorator));
            for hunk in self.hunks.iter().filter(|h| &h.file == file_name) {
                doc.push(elements::Paragraph::new(hunk.content.trim_end()));
                // Comments for this hunk
                for comment in self.comments.iter().filter(|c| c.contains(&hunk.id)) {
                    doc.push(elements::Paragraph::new(format!("> {}", comment)).styled(style::Style::new().with_color(style::Color::Rgb(128, 0, 0))))
                }
            }
        }
        doc.push(elements::Break::new(1));
        doc.push(elements::Paragraph::new("Comments:").styled(decorator));
        for comment in &self.comments {
            doc.push(elements::Paragraph::new(format!("- {}", comment)));
        }
        doc.push(elements::Break::new(1));
        doc.push(elements::Paragraph::new("Chat:").styled(decorator));
        for chat in &self.chat {
            doc.push(elements::Paragraph::new(format!("- {}", chat)));
        }
        let mut out = std::fs::File::create("reviewmesh_export.pdf")?;
        doc.render(&mut out)?;
        Ok(())
    }

    fn handle_input(&mut self) {
        let trimmed = self.input.trim();
        if self.input_mode == InputMode::Command {
            if trimmed == "/chat" {
                self.input_mode = InputMode::Chat;
                self.status = "Chat mode (Enter to send, / to enter command)".to_string();
            } else if trimmed.starts_with("/comment ") {
                let comment_body = trimmed[9..].to_string();
                let hunk = self.current_hunk();
                let (file, hunk_id, line) = if let Some(h) = hunk {
                    (h.file.clone(), h.id.clone(), h.new_start)
                } else {
                    ("".to_string(), "".to_string(), 0)
                };
                let comment = Comment {
                    id: uuid::Uuid::new_v4().to_string(),
                    session_id: self.session.id.clone(),
                    author: self.author.clone(),
                    file,
                    hunk_id,
                    line,
                    body: comment_body.clone(),
                    created_at: Utc::now(),
                    resolved: false,
                };
                self.comments.push(comment_body.clone());
                if let Some(tx) = &self.net_tx {
                    let _ = tx.send(NetMsg::Comment(comment.clone()));
                }
                if let Some(store) = &self.storage {
                    let _ = store.save_comment(&comment);
                }
                self.status = "Comment added and sent.".to_string();
            } else if trimmed == "/resolve" {
                if let Some(last) = self.comments.last_mut() {
                    *last = format!("[RESOLVED] {}", last);
                    self.status = "Last comment marked as resolved.".to_string();
                } else {
                    self.status = "No comments to resolve.".to_string();
                }
            } else if trimmed == "/export pdf" {
                match self.export_pdf() {
                    Ok(_) => self.status = "Exported to reviewmesh_export.pdf".to_string(),
                    Err(e) => self.status = format!("PDF export failed: {}", e),
                }
            } else if trimmed == "/export" {
                match self.export_markdown() {
                    Ok(_) => self.status = "Exported to reviewmesh_export.md".to_string(),
                    Err(e) => self.status = format!("Export failed: {}", e),
                }
            } else if trimmed == "/help" {
                self.status = "Commands: /comment <text>, /resolve, /chat, /export, /export pdf, /help, q".to_string();
            } else if trimmed == "q" {
                // Will be handled in main loop
            } else {
                self.status = format!("Unknown command: {}", trimmed);
            }
            self.input.clear();
        } else {
            // Chat mode
            if trimmed.starts_with("/") {
                self.input_mode = InputMode::Command;
                self.status = "Command mode (type /help for commands)".to_string();
            } else if !trimmed.is_empty() {
                let chat_line = ChatLine {
                    id: uuid::Uuid::new_v4().to_string(),
                    session_id: self.session.id.clone(),
                    author: self.author.clone(),
                    body: trimmed.to_string(),
                    created_at: Utc::now(),
                };
                self.chat.push(trimmed.to_string());
                if let Some(tx) = &self.net_tx {
                    let _ = tx.send(NetMsg::Chat(chat_line.clone()));
                }
                if let Some(store) = &self.storage {
                    let _ = store.save_chat(&chat_line);
                }
                self.input.clear();
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup session and author (stub, could be CLI args)
    let session = ReviewSession {
        id: "default-session".to_string(),
        title: "Default Session".to_string(),
        created_at: Utc::now(),
        participants: vec![],
    };
    let author = whoami::username();
    // Setup storage
    let storage = Storage::new("reviewmesh.sqlite").ok();
    // Networking setup
    let mut network = Network::new(&session, b"reviewmesh-secret").await?;
    network.join_topic(&format!("reviewmesh-{}", session.id));
    network.join_topic(&format!("chatmesh-{}", session.id));
    // Channels for network <-> UI
    let (ui_tx, ui_rx) = mpsc::channel();
    let (net_tx, net_rx) = mpsc::channel();
    // Spawn network poller
    let session_id = session.id.clone();
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            use futures::future::poll_fn;
            let mut network = network;
            loop {
                poll_fn(|cx| {
                    match network.poll_next(cx) {
                        std::task::Poll::Ready(Some((topic, data))) => {
                            if topic.starts_with("chatmesh-") {
                                if let Ok(chat) = serde_json::from_slice::<ChatLine>(&data) {
                                    let _ = ui_tx.send(NetMsg::Chat(chat));
                                }
                            } else if topic.starts_with("reviewmesh-") {
                                if let Ok(comment) = serde_json::from_slice::<Comment>(&data) {
                                    let _ = ui_tx.send(NetMsg::Comment(comment));
                                }
                            }
                            std::task::Poll::Pending
                        }
                        std::task::Poll::Pending => std::task::Poll::Pending,
                        std::task::Poll::Ready(None) => std::task::Poll::Pending,
                    }
                }).await;
                // Send outgoing
                while let Ok(msg) = net_rx.try_recv() {
                    match msg {
                        NetMsg::Chat(chat) => {
                            let topic = format!("chatmesh-{}", session_id);
                            let _ = network.send_message(&topic, &serde_json::to_vec(&chat).unwrap());
                        }
                        NetMsg::Comment(comment) => {
                            let topic = format!("reviewmesh-{}", session_id);
                            let _ = network.send_message(&topic, &serde_json::to_vec(&comment).unwrap());
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });
    });
    // Setup TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new(session, author, storage);
    app.net_tx = Some(net_tx);
    let res = run_app(&mut terminal, &mut app, ui_rx);
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    if let Err(e) = res {
        eprintln!("Error: {}", e);
    }
    Ok(())
}

fn run_app<B: tui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App, ui_rx: Receiver<NetMsg>) -> io::Result<()> {
    loop {
        // Handle incoming network messages
        while let Ok(msg) = ui_rx.try_recv() {
            match msg {
                NetMsg::Chat(chat) => {
                    app.chat.push(format!("{}: {}", chat.author, chat.body));
                }
                NetMsg::Comment(comment) => {
                    app.comments.push(format!("{}: {}", comment.author, comment.body));
                }
            }
        }

        terminal.draw(|f| {
            let size = f.size();
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Session info
                    Constraint::Min(0),   // Main panes
                ].as_ref())
                .split(size);

            // Session info (top)
            let session_info = format!(
                "Session: {} | ID: {} | Participants: {}",
                app.session.title,
                app.session.id,
                app.session.participants.join(", ")
            );
            let session_block = Paragraph::new(session_info)
                .block(Block::default().title("Session Info").borders(Borders::ALL))
                .wrap(Wrap { trim: true });
            f.render_widget(session_block, vertical_chunks[0]);

            // Main panes (files, diff+comments, chat)
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30), // File list
                    Constraint::Percentage(30), // Diff + comments
                    Constraint::Percentage(40), // Chat
                ].as_ref())
                .split(vertical_chunks[1]);

            // File list (left)
            let file_items: Vec<ListItem> = app.files.iter().enumerate().map(|(i, file)| {
                let mut s = file.clone();
                if i == app.selected_file {
                    s = format!("> {} <", s);
                }
                ListItem::new(s)
            }).collect();
            let file_list = List::new(file_items)
                .block(Block::default().title("Files (Up/Down)").borders(Borders::ALL));
            f.render_widget(file_list, chunks[0]);

            // Diff + inline comments (middle)
            let mut diff_lines = vec![];
            if let Some(hunk) = app.current_hunk() {
                for line in hunk.content.lines() {
                    diff_lines.push(ListItem::new(line.to_string()));
                }
                // Inline comments for this hunk
                let hunk_comments: Vec<_> = app.comments.iter().filter(|c| c.contains(&hunk.id)).collect();
                if !hunk_comments.is_empty() {
                    diff_lines.push(ListItem::new("--- Comments ---"));
                    for c in hunk_comments {
                        diff_lines.push(ListItem::new(format!("[comment] {}", c)));
                    }
                }
            } else {
                diff_lines.push(ListItem::new("No diff"));
            }
            let diff_list = List::new(diff_lines)
                .block(Block::default().title("Diff (Left/Right hunks)").borders(Borders::ALL));
            f.render_widget(diff_list, chunks[1]);

            // Chat/comments (right)
            let chat: Vec<ListItem> = app.chat.iter().map(|c| ListItem::new(c.clone())).collect();
            let chat_list = List::new(chat)
                .block(Block::default().title("Chat").borders(Borders::ALL));
            f.render_widget(chat_list, chunks[2]);

            // Input bar (bottom)
            let input_rect = Rect {
                x: size.x,
                y: size.height - 3,
                width: size.width,
                height: 3,
            };
            let input_title = match app.input_mode {
                InputMode::Command => ":command",
                InputMode::Chat => ":chat",
            };
            let input_block = Block::default().title(input_title).borders(Borders::ALL);
            let input = Paragraph::new(app.input.as_ref())
                .style(Style::default().fg(Color::Yellow))
                .block(input_block);
            f.render_widget(input, input_rect);
        })?;

        // Event handling
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Char('q') if app.input_mode == InputMode::Command => return Ok(()),
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Enter => {
                        app.handle_input();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Chat;
                        app.status = "Chat mode (Enter to send, / to enter command)".to_string();
                        app.input.clear();
                    }
                    KeyCode::Up => {
                        if app.input_mode == InputMode::Command {
                            if app.selected_file > 0 {
                                app.selected_file -= 1;
                                app.selected_hunk = 0;
                            }
                        }
                    }
                    KeyCode::Down => {
                        if app.input_mode == InputMode::Command {
                            if app.selected_file + 1 < app.files.len() {
                                app.selected_file += 1;
                                app.selected_hunk = 0;
                            }
                        }
                    }
                    KeyCode::Left => {
                        if app.input_mode == InputMode::Command {
                            if app.selected_hunk > 0 {
                                app.selected_hunk -= 1;
                            }
                        }
                    }
                    KeyCode::Right => {
                        if app.input_mode == InputMode::Command {
                            let hunks = app.current_hunks();
                            if app.selected_hunk + 1 < hunks.len() {
                                app.selected_hunk += 1;
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
