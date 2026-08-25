<div align="center">

# HFS-RS ⚡

**基于 Rust 与 GPUI 构建的高性能现代 HTTP 文件服务器**  
*致敬经典 [HFS 2](https://github.com/rejetto/hfs2)，结合纯原生 GPU 硬件加速桌面渲染与异步高并发底层。*

<p align="center">
  <b><a href="README_zh.md">🇨🇳 简体中文</a></b> | <b><a href="README.md">🇺🇸 English</a></b>
</p>

[![CI](https://github.com/821869798/hfs-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/821869798/hfs-rs/actions/workflows/ci.yml)
[![Release](https://github.com/821869798/hfs-rs/actions/workflows/release.yml/badge.svg)](https://github.com/821869798/hfs-rs/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-informational.svg)]()

</div>

---

## 📖 项目简介

**HFS-RS** 是对经典 **HTTP File Server (HFS 2)** 的全方位现代化 Rust 重构版本。它保留了 HFS 2 经典的即开即用、零配置极速文件共享交互逻辑，同时利用现代异步 I/O 与原生 GPU 硬件加速渲染技术，带来了极致的吞吐性能、更低的资源开销与丝滑的现代化桌面交互。

无论是用于局域网超快文件分享、搭建轻量级 Web 文件直链服务，还是管理带账户权限的上传下载门户，**HFS-RS** 都能提供高并发、断点续传与直观易用的桌面控制体验。

---

## ✨ 核心特性

- 🌲 **虚拟文件系统 (VFS)**：
  - 自由混合组织磁盘真实目录、单个文件、虚拟文件夹层级结构以及外部网络链接。
  - 支持节点重命名、自定义权限、磁盘目录动态实时映射。
- ⚡ **高性能 HTTP/1.1 核心**：
  - 基于 **Tokio** 与 **Hyper** 的全异步高并发流式传输架构。
  - 完整支持 **HTTP Range（多线程断点续传）**，大文件分块高速下载。
  - 支持流式分块多文件上传与自定义单文件大小限制。
- 🎨 **现代原生 GPU 桌面界面**：
  - 基于 **Zed GPUI-CE** 纯原生 GPU 硬件加速框架渲染（Direct3D / Metal / Vulkan）。
  - 支持 **简易模式（Easy Mode）** 与 **专家模式（Expert Mode）** 自由切换。
  - 支持**各区域自由拖拽调整大小**（左右分栏与上下工作区分割线自由无级伸缩）。
- 🌓 **智能主题与系统语言跟随**：
  - 默认**自动跟随操作系统语言**（简体中文 / 英文）与**深色/浅色外观偏好**。
  - 独立卡片式**选项面板**与下拉选择菜单，配置即时生效并自动持久化。
- 📊 **实时流量与连接监视**：
  - 实时出入站速率波形流量图（速度 / 累计传输字节）。
  - 活跃连接监控列表，实时展示访问者 IP、文件、传输进度、实时速度与预估剩余时间 (ETA)。
  - 结构化实时日志视窗，支持按级别查看与一键快速复制。
- 🔐 **多用户安全认证与权限策略**：
  - 支持禁止上传、公开允许上传、需账号密码保护上传等多种安全策略。
  - 支持多用户账号增删改查与密码认证。

---

## 🚀 快速开始

### 下载预编译版本

从 **[GitHub Releases](https://github.com/821869798/hfs-rs/releases)** 下载最新独立运行包：
- 解压 `hfs-rs-vX.X.X-windows-x86_64-portable.zip` 并双击运行 `hfs-rs.exe` 即可使用。

### 源码编译

请确保本地已安装 [Rust](https://www.rust-lang.org/) 环境：

```bash
# 克隆代码仓库
git clone https://github.com/821869798/hfs-rs.git
cd hfs-rs

# 本地调试运行
cargo run

# 构建高度优化的 Release 二进制文件
cargo build --release --bin hfs-rs
```

编译出的单文件可执行程序位于 `target/release/hfs-rs.exe`（Windows）或 `target/release/hfs-rs`（Linux/macOS）。

---

## 🛠️ 技术栈与架构

| 模块 | 采用技术 | 说明 |
| :--- | :--- | :--- |
| **开发语言** | [Rust](https://www.rust-lang.org/) | 内存安全、零成本抽象与高并发 |
| **界面引擎** | [GPUI-CE](https://github.com/gpui-ce/gpui-ce) | Zed 抽离的高性能 GPU 硬件加速桌面渲染框架 |
| **异步运行时** | [Tokio](https://tokio.rs/) | 多线程非阻塞事件循环调度 |
| **HTTP 引擎** | [Hyper](https://hyper.rs/) | 工业级高性能 HTTP/1.1 服务端实现 |
| **国际化 (i18n)** | 内置轻量引擎 | 零外部依赖，支持系统语言自动识别与双语实时切换 |

更详细的技术设计文档请参考 [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) 与开发路线 [`docs/PROGRESS.md`](docs/PROGRESS.md)。

---

## 🤝 参与贡献

欢迎提交 Issue 报告 Bug 或提出新功能需求，也欢迎提交 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feat/my-feature`)
3. 提交修改 (`git commit -m 'feat: add some feature'`)
4. 推送到远程分支 (`git push origin feat/my-feature`)
5. 新建 Pull Request

---

## 📄 开源协议

本项目采用 [MIT License](LICENSE) 开源协议。
