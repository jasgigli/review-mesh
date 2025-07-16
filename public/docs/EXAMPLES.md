# ReviewMesh Usage Examples

## 1. Start a Review Session
```sh
./target/release/cli.exe review login-session --target-branch feature/login
```
- Starts a review session for the `feature/login` branch.

---

## 2. Join a Review Session (Second Developer)
```sh
./target/release/cli.exe review login-session --target-branch feature/login
```
- Joins the same session on the same network.

---

## 3. Add a Comment
Type in the TUI:
```
/comment Please refactor this function.
```
- Adds a comment to the first diff hunk.

---

## 4. Chat with Other Reviewers
Type in the TUI:
```
Great work!
```
- Sends a chat message to all participants.

---

## 5. Export the Review to PDF
```sh
./target/release/cli.exe export login-session login-review.pdf
```
- Exports the session to `login-review.pdf`.

---

## 6. Troubleshooting
- **Fonts missing for PDF export:**
  - Download LiberationSans fonts and place them in a `fonts` directory in your project root.
- **Networking:**
  - All participants must be on the same local network for auto-discovery.

---

## 7. See All Commands
```sh
./target/release/cli.exe --help
```

---

Happy reviewing!
