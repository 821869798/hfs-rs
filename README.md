# HFS-RS

Rust 版 [HFS 2](https://github.com/rejetto/hfs2)（HTTP File Server）。

界面与功能目标对齐 HFS2，技术栈：

- **Rust**
- **GPUI** + **gpui-component**（参考 `pass_smash`）
- **hyper / tokio** 提供 HTTP 服务

## 文档

- [产品规划](docs/PLAN.md)
- [开发进度](docs/PROGRESS.md)
- [架构说明](docs/ARCHITECTURE.md)

## 开发

```bash
cargo run
```

默认监听 `0.0.0.0:8080`。

## 当前能力（M0/M1 进行中）

- 主窗口分区：工具栏 / VFS / 连接 / 日志 / 状态栏
- 添加文件与文件夹到虚拟文件系统
- 启动/停止 HTTP 服务
- 浏览器目录浏览与文件下载（Range）
- 基础日志与连接信息

## 许可证

MIT
