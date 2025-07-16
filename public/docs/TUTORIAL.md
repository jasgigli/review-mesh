# ReviewMesh Tutorial

## Overview
ReviewMesh is a peer-to-peer code review tool for distributed teams. It enables real-time, collaborative review sessions with chat, inline comments, and PDF export.

---

## Scenario: Two Developers Collaborating

**Alice** creates a feature branch and pushes code. **Bob** reviews it. Both can comment, chat, and export the review.

---

### 1. Setup

Both developers clone the repo and build ReviewMesh:
```sh
cargo build --release
```

---

### 2. Alice Creates a Feature Branch
```sh
git checkout -b feature/cool-feature
echo "fn cool() { println!(\"Cool!\"); }" > cool.rs
git add cool.rs
git commit -m "feat: add cool feature"
```

---

### 3. Bob Prepares to Review
Bob switches to `main` and starts a review session:
```sh
git checkout main
./target/release/cli.exe review cool-session --target-branch feature/cool-feature
```
- The TUI opens. Diffs are shown on the left, comments on the right.

---

### 4. Alice Joins the Review
Alice (on her own terminal or machine, same network) runs:
```sh
./target/release/cli.exe review cool-session --target-branch feature/cool-feature
```
- Both are now in the same session.

---

### 5. Commenting and Chat
- To comment on the first hunk:
  ```
  /comment Please add a test for this function.
  ```
- To chat:
  ```
  Looks good!
  ```
- All comments and chat are synced in real time.

---

### 6. Exiting the Review
- Press `Esc` to exit the TUI.
- All data is saved in `review_mesh.db`.

---

### 7. Exporting the Review
Either developer can export the session:
```sh
./target/release/cli.exe export cool-session cool-review.pdf
```
- The PDF will include diffs, comments, and chat.

---

### 8. Tips & Collaboration
- **Session ID:** Use the same session ID for all participants.
- **Network:** All participants must be on the same local network for auto-discovery.
- **Fonts:** For PDF export, place LiberationSans fonts in a `fonts` directory in your project root.
- **Multiple Sessions:** You can have multiple review sessions for different branches.

---

## Example Workflow

```sh
# Alice
$ git checkout -b feature/cool-feature
$ echo "fn cool() { println!(\"Cool!\"); }" > cool.rs
$ git add cool.rs
$ git commit -m "feat: add cool feature"

# Bob
$ git checkout main
$ ./target/release/cli.exe review cool-session --target-branch feature/cool-feature

# Alice (joins review)
$ ./target/release/cli.exe review cool-session --target-branch feature/cool-feature

# Both comment and chat in the TUI

# Export
$ ./target/release/cli.exe export cool-session cool-review.pdf
```

---

## Need Help?
Run:
```sh
./target/release/cli.exe --help
```

---

Happy reviewing with ReviewMesh!
