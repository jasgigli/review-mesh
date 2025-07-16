use git2::{Repository, DiffOptions, DiffHunk as Git2Hunk, DiffLine, Oid};
use common::DiffHunk;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::cell::RefCell;

pub fn compute_diff(repo_path: &str, target_branch: &str) -> Vec<DiffHunk> {
    let repo = match Repository::open(repo_path) {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    let head = match repo.head().and_then(|h| h.peel_to_commit()) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let target = match repo.revparse_single(target_branch).and_then(|o| o.peel_to_commit()) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut diff_opts = DiffOptions::new();
    let diff = match repo.diff_tree_to_tree(
        Some(&target.tree().unwrap()),
        Some(&head.tree().unwrap()),
        Some(&mut diff_opts),
    ) {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    struct State {
        file_path: String,
        hunk_content: String,
        old_start: usize,
        old_lines: usize,
        new_start: usize,
        new_lines: usize,
        in_hunk: bool,
        hunks: Vec<DiffHunk>,
    }
    let state = RefCell::new(State {
        file_path: String::new(),
        hunk_content: String::new(),
        old_start: 0,
        old_lines: 0,
        new_start: 0,
        new_lines: 0,
        in_hunk: false,
        hunks: vec![],
    });
    diff.foreach(
        &mut |file, _| {
            let mut s = state.borrow_mut();
            s.file_path = file.new_file().path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            true
        },
        None,
        Some(&mut |_, hunk| {
            let mut s = state.borrow_mut();
            if s.in_hunk {
                let id = hunk_id(&s.file_path, s.old_start, s.new_start, &s.hunk_content);
                // Extract values before pushing to avoid double borrow
                let file = s.file_path.clone();
                let old_start = s.old_start;
                let old_lines = s.old_lines;
                let new_start = s.new_start;
                let new_lines = s.new_lines;
                let content = s.hunk_content.clone();
                s.hunks.push(DiffHunk {
                    id,
                    file,
                    old_start,
                    old_lines,
                    new_start,
                    new_lines,
                    content,
                });
                s.hunk_content.clear();
            }
            s.in_hunk = true;
            s.old_start = hunk.old_start() as usize;
            s.old_lines = hunk.old_lines() as usize;
            s.new_start = hunk.new_start() as usize;
            s.new_lines = hunk.new_lines() as usize;
            s.hunk_content.push_str(&format!("@@ -{},{} +{},{} @@\n", s.old_start, s.old_lines, s.new_start, s.new_lines));
            true
        }),
        Some(&mut |_, _, line| {
            let mut s = state.borrow_mut();
            if s.in_hunk {
                s.hunk_content.push(line.origin() as char);
                s.hunk_content.push_str(&String::from_utf8_lossy(line.content()));
            }
            true
        }),
    ).unwrap_or(());
    let mut s = state.borrow_mut();
    if s.in_hunk && !s.hunk_content.is_empty() {
        let id = hunk_id(&s.file_path, s.old_start, s.new_start, &s.hunk_content);
        // Extract values before pushing to avoid double borrow
        let file = s.file_path.clone();
        let old_start = s.old_start;
        let old_lines = s.old_lines;
        let new_start = s.new_start;
        let new_lines = s.new_lines;
        let content = s.hunk_content.clone();
        s.hunks.push(DiffHunk {
            id,
            file,
            old_start,
            old_lines,
            new_start,
            new_lines,
            content,
        });
    }
    s.hunks.clone()
}

pub fn hunk_id(file: &str, old_start: usize, new_start: usize, content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    file.hash(&mut hasher);
    old_start.hash(&mut hasher);
    new_start.hash(&mut hasher);
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
