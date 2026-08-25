<div align="center">

# HFS-RS ⚡

**A High-Performance, Modern HTTP File Server in Rust with GPUI**  
*Inspired by the classic [HFS 2](https://github.com/rejetto/hfs2), powered by cutting-edge native GPU desktop rendering.*

<p align="center">
  <b><a href="README_zh.md">🇨🇳 简体中文</a></b> | <b><a href="README.md">🇺🇸 English</a></b>
</p>

[![CI](https://github.com/821869798/hfs-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/821869798/hfs-rs/actions/workflows/ci.yml)
[![Release](https://github.com/821869798/hfs-rs/actions/workflows/release.yml/badge.svg)](https://github.com/821869798/hfs-rs/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-informational.svg)]()

</div>

---

## 📖 Overview

**HFS-RS** is a complete, modern rewrite of the classic **HTTP File Server (HFS 2)** built from the ground up in **Rust**. It brings the timeless simplicity of instant file sharing together with state-of-the-art asynchronous I/O and hardware-accelerated GPU desktop rendering.

Whether you want to quickly share files across your local network, deploy a lightweight web directory, or manage an authenticated download/upload portal, **HFS-RS** delivers high throughput, low memory footprint, and an intuitive UI.

---

## ✨ Features

- 🌲 **Virtual File System (VFS)**:
  - Freely mix real disk folders, single files, virtual hierarchical folders, and web links.
  - Granular access control, custom display names, and real-time disk tree synchronization.
- ⚡ **High Throughput HTTP/1.1 Engine**:
  - Asynchronous multi-threaded streaming powered by **Tokio** & **Hyper**.
  - Full **HTTP Range** support for multithreaded downloads and resumable transfers.
  - Streaming multipart uploads with configurable size limits.
- 🎨 **Modern Native GPU Desktop UI**:
  - Built on **Zed's GPUI-CE** engine with Direct3D/Metal hardware acceleration.
  - Fully customizable **Easy Mode** & **Expert Mode** layout.
  - Responsive **Draggable Splitters** for resizing sidebars, logs, and connection monitors.
- 🌓 **Adaptive Themes & System Following**:
  - Auto-detection for OS language (Chinese / English) and Dark/Light theme preference.
  - Card-based independent **Options View** with instant dropdown configurations.
- 📊 **Real-Time Monitoring**:
  - Live bandwidth traffic graph (Inbound & Outbound speed / total bytes).
  - Real-time active connection tracking with IP, speed, progress, and ETA.
  - Structured application logs with one-click clipboard copying.
- 🔐 **Security & Multi-User Management**:
  - Public uploads, account-protected uploads, and custom HTTP authentication.
  - Configurable server port, connection limits, and auto-URL copying on drag-and-drop.

---

## 🚀 Quick Start

### Download Prebuilt Binaries

Download the latest standalone executable from **[GitHub Releases](https://github.com/821869798/hfs-rs/releases)**:
- Extract `hfs-rs-vX.X.X-windows-x86_64-portable.zip` and run `hfs-rs.exe`.

### Build From Source

Make sure you have [Rust](https://www.rust-lang.org/) installed (Rust 2024 edition supported):

```bash
# Clone the repository
git clone https://github.com/821869798/hfs-rs.git
cd hfs-rs

# Run debug build
cargo run

# Build optimized release binary
cargo build --release --bin hfs-rs
```

Binary will be located at `target/release/hfs-rs` (or `hfs-rs.exe` on Windows).

---

## 🛠️ Architecture & Tech Stack

| Component | Technology | Description |
| :--- | :--- | :--- |
| **Language** | [Rust](https://www.rust-lang.org/) | Memory safe, zero-cost abstractions, fearless concurrency |
| **UI Engine** | [GPUI-CE](https://github.com/gpui-ce/gpui-ce) | Hardware-accelerated GPU desktop rendering framework |
| **Async Runtime** | [Tokio](https://tokio.rs/) | Multi-threaded non-blocking asynchronous event loop |
| **HTTP Engine** | [Hyper](https://hyper.rs/) | Fast and correct HTTP/1.1 server implementation |
| **Internationalization** | Builtin `i18n` | Zero-dependency bilingual engine with OS language detection |

Detailed architectural diagrams and roadmaps can be found in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) and [`docs/PROGRESS.md`](docs/PROGRESS.md).

---

## 🤝 Contributing

Contributions, bug reports, and feature suggestions are welcome!

1. Fork the repository
2. Create your feature branch (`git checkout -b feat/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add some amazing feature'`)
4. Push to the branch (`git push origin feat/amazing-feature`)
5. Open a Pull Request

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
