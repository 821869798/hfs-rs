# HFS-RS 架构说明

## 1. 仓库结构

```
hfs-rs/
├── Cargo.toml
├── README.md
├── docs/
│   ├── PLAN.md           # 产品规划与里程碑
│   ├── PROGRESS.md       # 开发进度
│   └── ARCHITECTURE.md   # 本文件
├── templates/
│   └── directory.html    # 目录列表模板
├── assets/               # 图标等静态资源（后续）
└── src/
    ├── main.rs           # 程序入口
    ├── lib.rs            # 库根，便于测试
    ├── config/
    │   └── mod.rs        # AppConfig
    ├── vfs/
    │   └── mod.rs        # 虚拟文件系统
    ├── http/
    │   └── mod.rs        # 请求处理、MIME、Range、HTML
    ├── server/
    │   └── mod.rs        # ServerHandle 生命周期与统计
    ├── ui/
    │   ├── mod.rs
    │   └── app.rs        # 主窗口
    └── util/
        └── mod.rs        # 格式化工具
```

## 2. 运行时模型

```
main
 └─ gpui_platform::application
     └─ open_window(HfsApp)
         ├─ UI state (选中节点、日志视图、模式…)
         ├─ shared AppState (config + vfs + server)
         └─ poll loop (50ms)
              └─ drain server events → update UI
```

HTTP 服务在独立 `tokio` runtime 线程中运行：

```
ServerHandle::start()
  └─ thread::spawn
       └─ tokio runtime
            └─ hyper bind/accept loop
                 └─ per-request: clone state → handle
```

停止服务：`CancellationToken` / oneshot + `shutdown`，UI 收到 `ServerStopped`。

## 3. 共享状态

```rust
AppState {
  config: Arc<RwLock<AppConfig>>,
  vfs: Arc<RwLock<Vfs>>,
  server: ServerHandle,           // 内含 connections / traffic
  bus: EventBus,                  // UI 可 poll 的消息队列
}
```

### 事件

- `Log { level, text }`
- `ConnectionAdded/Updated/Removed`
- `ServerStarted { addr }`
- `ServerStopped`
- `Stats { bytes_in, bytes_out, connections }`
- `BandwidthSample { up, down }`（后续）

## 4. VFS

- 以 `NodeId(u64)` 为稳定 ID
- 根节点恒为 VirtualFolder `/`
- **VirtualFolder**：只包含显式子节点
- **RealFolder**：子项来自磁盘列举 + 可附加虚拟子节点
- **File**：映射到本地文件
- **Link**：重定向 URL（P1）

URL 解析：

1. percent-decode path
2. 按 `/` 分段
3. 从 root 向下查找 name
4. RealFolder 未显式挂载的名字 → 尝试磁盘拼接
5. 权限检查（P1）

## 5. HTTP 处理管线

```
request
  → 解析 path / method
  → 特殊路径 (~style, favicon…)
  → vfs.resolve
  → method dispatch
       GET dir  → render template
       GET file → open + Range + stream
       PUT/POST → upload (P1)
  → 更新 conn stats + log
```

## 6. UI 模块

`HfsApp` 负责：

- 工具栏操作（start/stop、模式、端口）
- VFS 操作（add/remove/rename）并 `cx.notify`
- 连接表渲染
- 日志渲染
- 定时 poll `EventBus`

子视图可按文件拆分（后续）：

- `toolbar.rs`
- `vfs_pane.rs`
- `conn_pane.rs`
- `log_pane.rs`
- `graph.rs`

P0 为降低跳跃成本，先集中在 `ui/app.rs`。

## 7. 配置

`AppConfig` 字段（初期）：

- `port: u16`（默认 8080）
- `bind: String`（默认 `0.0.0.0`）
- `expert_mode: bool`
- `auto_copy_url: bool`
- `server_id_header: bool`
- `max_connections: Option<usize>`

存储：`hfs-rs.json`（可执行文件旁或用户配置目录）。

## 8. 错误处理

- 库内用 `anyhow` / `thiserror`（核心用 thiserror，边界 anyhow）
- HTTP 错误映射为状态码页面
- UI 操作失败写入日志面板，不 panic
- `main` 安装 panic hook 写临时崩溃日志（同 pass_smash）

## 9. 测试策略

- 单元：VFS resolve、Range parse、config serde
- 集成：用 `hyper` client 打本地 server（绑定 `127.0.0.1:0`）
- UI：手工测试清单见 PROGRESS

## 10. 与 HFS2 源码对应

| HFS2 | hfs-rs |
|------|--------|
| `Tfile` | `vfs::VfsNode` |
| `TconnData` | `server::ConnInfo` |
| `Taccount` | `config::Account`（P1）
| `main.pas` GUI | `ui/app.rs` |
| ICS HTTP | `http` + `server` + hyper |
| `default.tpl` | `templates/*` + 未来宏引擎 |
| `optionsDlg` | 未来 `ui/options.rs` |
