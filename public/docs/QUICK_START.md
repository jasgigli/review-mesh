# ReviewMesh Quick Start Guide

Welcome to **ReviewMesh**! This guide will help you get up and running with collaborative code reviews in minutes.

---

## 1. Build the Project

```sh
cargo build --release
```

---

## 2. Create a Feature Branch (Developer 1)

Simulate a code change to review:
```sh
git checkout -b feature/my-feature
# Make some changes, e.g.:
echo "Hello ReviewMesh!" > hello.txt
git add hello.txt
git commit -m "feat: add hello.txt"
```

---

## 3. Switch to Main and Start a Review Session (Reviewer)

```sh
git checkout main
./target/release/cli.exe review my-session --target-branch feature/my-feature
```
- This opens the ReviewMesh TUI.
- The left pane shows diffs, the right pane shows comments.

---

## 4. Join the Review (Developer 2, on another terminal or machine)

On the same network, run:
```sh
./target/release/cli.exe review my-session --target-branch feature/my-feature
```
- Both developers are now in the same review session.
- Comments and chat are synced in real time.

---

## 5. Add Comments and Chat
- To comment on the first hunk:
  ```
  /comment Please add more tests.
  ```
- To chat:
  ```
  Looks good to me!
  ```
- Press `Esc` to exit the TUI.

---

## 6. Export the Review to PDF

```sh
./target/release/cli.exe export my-session review.pdf
```
- The file `review.pdf` will contain the session title, diffs, comments, and chat.

---

## 7. Troubleshooting
- **Fonts required for PDF export:**
  - Download LiberationSans fonts and place them in a `fonts` directory in your project root.
- **Networking:**
  - All participants must be on the same local network for auto-discovery.

---

## 8. Help

See all commands:
```sh
./target/release/cli.exe --help
```

---

Happy reviewing!
