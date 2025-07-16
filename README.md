# ReviewMesh

[![Build Status](https://github.com/your-org/review-mesh/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/review-mesh/actions)
[![Crates.io](https://img.shields.io/crates/v/reviewmesh.svg)](https://crates.io/crates/reviewmesh)
[![Docs.rs](https://docs.rs/reviewmesh/badge.svg)](https://docs.rs/reviewmesh)

> **ReviewMesh**: Peer-to-peer code review in your terminal. No servers. No friction. Just code, comments, and chat—anywhere, anytime.

---

## Features

- 🌐 **P2P code review**: Start or join sessions over libp2p, no central server
- 🧑‍💻 **Git diff TUI**: View diffs, leave inline comments, mark resolved
- 💬 **Real-time chat**: Split-pane chat alongside code review
- 📴 **Offline-first**: Comments and chat persist to SQLite, sync on reconnect
- 📤 **Export**: Save reviews as Markdown or PDF
- 🔒 **Secure**: Noise encryption, invite tokens
- 🛠️ **Open source**: MIT/Apache-2.0, contributions welcome!

---

## Screenshots

> _TUI screenshots and GIFs coming soon_

![screenshot placeholder](docs/screenshot.png)

---

## Getting Started

```sh
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
$ git clone https://github.com/your-org/review-mesh.git
$ cd review-mesh
$ cargo build --release

# Start a review session
$ cargo run --release --bin cli start
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

---

## License

Licensed under MIT or Apache-2.0, at your option.

---

## Demo Video

[![Getting Started Video](https://img.shields.io/badge/YouTube-Video-red)](https://youtu.be/your-demo-link)
