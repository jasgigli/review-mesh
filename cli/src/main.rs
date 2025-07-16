use clap::{self, Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::{Duration, Instant}};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use uuid::Uuid;
use whoami;
use genpdf::{self, elements, style, Element};
use futures::StreamExt;
use serde_json;
use chrono::Utc;

use common::{ReviewSession, Comment, ChatLine, DiffHunk};
use storage::Storage;
use network::NetworkManager;
use git_integration::compute_diff;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Review {
        session_id: String,
        #[arg(short, long)]
        target_branch: Option<String>,
    },
    Export {
        session_id: String,
        file_path: String,
    },
}

struct App {
    storage: Storage,
    session: ReviewSession,
    hunks: Vec<DiffHunk>,
    comments: Vec<Comment>,
    chat_history: Vec<ChatLine>,
    network: NetworkManager,
}

impl App {
    fn new(storage: Storage, session_id: String, target_branch: Option<String>) -> Self {
        let session = storage.get_session(&session_id).unwrap().unwrap_or_else(|| {
            let new_session = ReviewSession {
                id: session_id.clone(),
                title: format!("Review for {}", session_id),
                created_at: Utc::now(),
                participants: vec![whoami::username()],
            };
            storage.save_session(&new_session).unwrap();
            new_session
        });

        let hunks = if let Some(branch) = target_branch {
            compute_diff(".", &branch)
        } else {
            vec![]
        };

        let comments = storage.get_comments(&session_id).unwrap();
        let chat_history = storage.get_chat_history(&session_id).unwrap();
        let network = NetworkManager::new(None).unwrap();

        Self {
            storage,
            session,
            hunks,
            comments,
            chat_history,
            network,
        }
    }

    fn on_tick(&mut self) {
        // Here we can poll the network for new messages
        while let Ok(Some(_event)) = self.network.swarm.try_poll_next() {
            // Process network events
        }
    }

    fn ui(&self, f: &mut Frame<impl Backend>, _input: &str) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        let hunks: Vec<ListItem> = self
            .hunks
            .iter()
            .map(|h| ListItem::new(Spans::from(h.content.clone())))
            .collect();
        let hunks_list = List::new(hunks)
            .block(Block::default().borders(Borders::ALL).title("Diff Hunks"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        f.render_widget(hunks_list, chunks[0]);

        let comments: Vec<ListItem> = self
            .comments
            .iter()
            .map(|c| ListItem::new(Spans::from(format!("{}: {}", c.author, c.body))))
            .collect();
        let comments_list = List::new(comments)
            .block(Block::default().borders(Borders::ALL).title("Comments"));
        f.render_widget(comments_list, chunks[1]);
    }

    fn export_to_pdf(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let font_family = genpdf::fonts::from_files("./fonts", "LiberationSans", None)
            .expect("Failed to load font family");
        let mut doc = genpdf::Document::new(font_family);
        doc.set_title("ReviewMesh Export");

        let mut decorator = genpdf::style::Style::new();
        decorator.set_bold();

        doc.push(elements::Paragraph::new(format!("ReviewMesh Export: {}", self.session.title)).with_style(decorator));

        for (file_name, hunks) in self.group_hunks_by_file() {
            let mut decorator = genpdf::style::Style::new();
            decorator.set_bold();
            doc.push(elements::Paragraph::new(format!("File: {}", file_name)).with_style(decorator));

            for hunk in hunks {
                doc.push(elements::Paragraph::new(hunk.content.clone()));

                let comments_for_hunk: Vec<&Comment> = self.comments.iter().filter(|c| c.hunk_id == hunk.id).collect();
                for comment in comments_for_hunk {
                    doc.push(elements::Paragraph::new(format!("> {}", comment.body)).with_style(style::Style::new().with_font_size(10)));
                }
            }
        }

        let mut decorator = genpdf::style::Style::new();
        decorator.set_bold();
        doc.push(elements::Paragraph::new("Comments:").with_style(decorator));
        for comment in &self.comments {
            doc.push(elements::Paragraph::new(format!("{}: {}", comment.author, comment.body)));
        }

        let mut decorator = genpdf::style::Style::new();
        decorator.set_bold();
        doc.push(elements::Paragraph::new("Chat:").with_style(decorator));
        for chat in &self.chat_history {
            doc.push(elements::Paragraph::new(format!("{}: {}", chat.author, chat.body)));
        }

        doc.render_to_file(file_path)?;
        Ok(())
    }

    fn group_hunks_by_file(&self) -> std::collections::HashMap<String, Vec<&DiffHunk>> {
        let mut map = std::collections::HashMap::new();
        for hunk in &self.hunks {
            map.entry(hunk.file.clone()).or_default().push(hunk);
        }
        map
    }

    fn handle_input(&mut self, input: &str) {
        if let Some(comment) = input.strip_prefix("/comment ") {
            if let Some(selected_hunk) = self.hunks.get(0) { // Simple: comment on the first hunk
                let new_comment = Comment {
                    id: Uuid::new_v4().to_string(),
                    session_id: self.session.id.clone(),
                    author: whoami::username(),
                    file: selected_hunk.file.clone(),
                    hunk_id: selected_hunk.id.clone(),
                    line: 0, // Placeholder
                    body: comment.to_string(),
                    created_at: Utc::now(),
                    resolved: false,
                };
                self.storage.save_comment(&new_comment).unwrap();
                self.network.publish_comment(&new_comment);
                self.comments.push(new_comment);
            }
        } else {
            let chat_line = ChatLine {
                id: Uuid::new_v4().to_string(),
                session_id: self.session.id.clone(),
                author: whoami::username(),
                body: input.to_string(),
                created_at: Utc::now(),
            };
            self.storage.save_chat_line(&chat_line).unwrap();
            // self.network.publish_chat(&chat_line);
            self.chat_history.push(chat_line);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Review { session_id, target_branch } => {
            let storage = Storage::new("review_mesh.db")?;
            let mut app = App::new(storage, session_id, target_branch);

            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            let tick_rate = Duration::from_millis(250);
            let mut last_tick = Instant::now();
            let mut input = String::new();

            loop {
                terminal.draw(|f| app.ui(f, &input))?;

                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if crossterm::event::poll(timeout)? {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Enter => {
                                app.handle_input(&input);
                                input.clear();
                            }
                            KeyCode::Char(c) => {
                                input.push(c);
                            }
                            KeyCode::Backspace => {
                                input.pop();
                            }
                            KeyCode::Esc => {
                                break;
                            }
                            _ => {}
                        }
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    app.on_tick();
                    last_tick = Instant::now();
                }
            }

            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
        }
        Commands::Export { session_id, file_path } => {
            let storage = Storage::new("review_mesh.db")?;
            let app = App::new(storage, session_id, None);
            app.export_to_pdf(&file_path)?;
            println!("Exported to {}", file_path);
        }
    }

    Ok(())
}
