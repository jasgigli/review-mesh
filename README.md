<div align="center">
  <h1>ReviewMesh</h1>
  <p>
    <strong>Peer-to-peer code review in your terminal. No servers. No friction.</strong>
  </p>
  <p>
    <strong>Just code, comments, and chat—anywhere, anytime.</strong>
  </p>
  <br />
  <p>
    <a href="https://github.com/your-org/review-mesh/actions/workflows/ci.yml">
      <img alt="Build Status" src="https://github.com/your-org/review-mesh/actions/workflows/ci.yml/badge.svg" />
    </a>
    <a href="https://crates.io/crates/reviewmesh">
      <img alt="Crates.io" src="https://img.shields.io/crates/v/reviewmesh.svg" />
    </a>
    <a href="https://docs.rs/reviewmesh">
      <img alt="Docs.rs" src="https://docs.rs/reviewmesh/badge.svg" />
    </a>
    <a href="https://github.com/your-org/review-mesh/blob/main/LICENSE">
      <img alt="License" src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" />
    </a>
  </p>
</div>

---

## 🚀 See it in Action

Bring your code reviews to life with a terminal-based UI that's fast, intuitive, and always available.

<div align="center">
  <!-- TODO: Replace with actual animated GIFs -->
  <img src="public/docs/placeholder-diff-view.gif" alt="Animated Diff View" width="70%" />
  <p><em>Inline commenting and real-time chat in the TUI.</em></p>
</div>

## ✨ Features

-   🌐 **Decentralized & P2P**: Start or join review sessions over libp2p. No central server, no bottlenecks.
-   🧑‍💻 **Interactive TUI**: A terminal UI for viewing diffs, leaving inline comments, and marking threads as resolved.
-   💬 **Real-Time Chat**: A dedicated chat panel for real-time discussion alongside your code.
-   📴 **Offline-First**: All comments and messages are saved locally to SQLite and sync automatically when you reconnect.
-   📤 **Flexible Export**: Save review sessions as Markdown or PDF for easy archiving and sharing.
-   🔒 **Secure by Default**: End-to-end encryption with the Noise protocol and secure invite tokens.
-   🛠️ **Open Source**: Licensed under MIT & Apache-2.0. Contributions are welcome!

---

## 📦 Installation

1.  **Install Rust**: If you don't have the Rust toolchain installed, get it from [rustup.rs](https://rustup.rs/).
    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2.  **Clone & Build**: Clone the repository and build the release binary.
    ```sh
    git clone https://github.com/your-org/review-mesh.git
    cd review-mesh
    cargo build --release
    ```
    The executable will be available at `target/release/review-mesh`.

---

## ⚙️ Usage

### To Start a Review Session

1.  Navigate to your Git repository.
2.  Run the `start` command and share the invite token with your peer.
    ```sh
    /path/to/review-mesh start
    ```

### To Join a Review Session

1.  Navigate to the same Git repository on your machine.
2.  Run the `join` command with the invite token from your peer.
    ```sh
    /path/to/review-mesh join <invite-token>
    ```

---

## 🤝 Contributing

We welcome contributions of all kinds! Please read our **[Contributing Guidelines](public/docs/CONTRIBUTING.md)** and **[Code of Conduct](public/docs/CODE_OF_CONDUCT.md)** to get started.

---

## 📜 License

This project is dual-licensed under either the [MIT License](LICENSE-MIT) or [Apache License, Version 2.0](LICENSE-APACHE) at your option.
