use git2::{Repository, DiffOptions, DiffHunk as Git2Hunk, DiffLine, Oid};
use common::DiffHunk;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
    let mut hunks = vec![];
    diff.foreach(
        &mut |file, _| {
            let file_path = file.new_file().path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            let mut hunk_idx = 0;
            let mut hunk_content = String::new();
            let mut old_start = 0;
            let mut old_lines = 0;
            let mut new_start = 0;
            let mut new_lines = 0;
            let mut in_hunk = false;
            diff.foreach_hunk(
                &mut |hunk| {
                    if in_hunk {
                        // Save previous hunk
                        let id = hunk_id(&file_path, old_start, new_start, &hunk_content);
                        hunks.push(DiffHunk {
                            id,
                            file: file_path.clone(),
                            old_start,
                            old_lines,
                            new_start,
                            new_lines,
                            content: hunk_content.clone(),
                        });
                        hunk_content.clear();
                    }
                    in_hunk = true;
                    old_start = hunk.old_start() as usize;
                    old_lines = hunk.old_lines() as usize;
                    new_start = hunk.new_start() as usize;
                    new_lines = hunk.new_lines() as usize;
                    hunk_content.push_str(&format!("@@ -{},{} +{},{} @@\n", old_start, old_lines, new_start, new_lines));
                    true
                },
                Some(&mut |line| {
                    if in_hunk {
                        hunk_content.push(line.origin() as char);
                        hunk_content.push_str(&String::from_utf8_lossy(line.content()));
                    }
                    true
                }),
            ).unwrap_or(());
            if in_hunk && !hunk_content.is_empty() {
                let id = hunk_id(&file_path, old_start, new_start, &hunk_content);
                hunks.push(DiffHunk {
                    id,
                    file: file_path.clone(),
                    old_start,
                    old_lines,
                    new_start,
                    new_lines,
                    content: hunk_content.clone(),
                });
            }
            true
        },
        None,
        None,
        None,
    ).unwrap_or(());
    hunks
}

pub fn hunk_id(file: &str, old_start: usize, new_start: usize, content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    file.hash(&mut hasher);
    old_start.hash(&mut hasher);
    new_start.hash(&mut hasher);
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
