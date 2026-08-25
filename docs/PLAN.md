# HFS-RS 规划文档

> 目标：用 Rust + GPUI + gpui-component 实现与 [HFS 2](https://github.com/rejetto/hfs2) 界面与功能基本一致的 HTTP 文件服务器。
>
> 参考：
> - 功能对标：`https://github.com/rejetto/hfs2`（本地参考源码：`D:\program\rs\hfs2-ref`）
> - GPUI 用法参考：`D:\program\rs\pass_smash`
> - 更细节 API：GPUI / Zed 源码与 longbridge/gpui-component

---

## 1. 产品定位

HFS（HTTP File Server）是“开箱即用”的个人文件分享工具：

- 本地 GUI 管理虚拟文件系统（VFS）
- 内置 HTTP 服务，浏览器即可下载/浏览/上传
- 支持账号权限、限速、日志、连接监控
- 与传统网盘/Web 服务器不同：轻量、可视化、单 exe 可运行

本项目 `hfs-rs` 是 HFS2 的 Rust 重写版，**不是 HFS3（Node.js）**，优先复刻 HFS2 的桌面体验与核心能力。

---

## 2. 对标范围（HFS2 核心）

### 2.1 主界面（必须高度一致）

| 区域 | HFS2 控件 | hfs-rs 目标 |
|------|-----------|-------------|
| 顶部工具栏 | Menu / Port / Easy-Expert / ON-OFF | 同布局按钮与状态 |
| URL 栏 | `urlBox` + 浏览器打开 + 复制 | 同功能 |
| 带宽图 | `graphBox` 实时绘制 | 简化但可用的带宽折线 |
| 虚拟文件系统 | `filesBox` TreeView | 树形 VFS 列表 + 右键菜单 |
| 连接列表 | `connBox` ListView | IP / File / Status / Speed / Progress |
| 日志面板 | `logBox` RichEdit | 彩色/分级日志 + 搜索 |
| 状态栏 | `sbar` | 连接数、流量、端口等 |

### 2.2 右键/菜单能力（第一期覆盖常用项）

- Add files / Add folder from disk
- New empty folder / New link
- Remove / Rename
- Copy URL
- Properties（基础属性）
- Start/Stop server
- Easy / Expert mode 切换
- Options 入口（分阶段实现）

### 2.3 服务端核心能力

**P0（第一可运行版本）**

- 监听可配置端口（默认 80/8080）
- VFS：虚拟根、真实文件、真实目录、虚拟目录
- 目录列表页（内置简单 HTML 模板）
- 文件下载（支持 Range 断点续传）
- 基础 MIME
- 连接状态回写 GUI
- 启动/停止服务
- 日志

**P1（功能对齐）**

- 账号系统（user/pass）与资源级权限（access/upload/delete）
- 上传
- 文件夹打包下载（zip/tar）
- 限速 / 最大并发下载
- IP ban / 白名单
- 自定义模板（兼容 HFS2 基础宏的子集）
- VFS 持久化（`.vfs` / JSON）
- 配置持久化

**P2（深度对齐）**

- 宏模板引擎（`{.xxx.}`）完整子集
- diff template
- 图标系统 / tray
- shell 右键集成
- DoS 防护、会话登录、IPv6
- 专家模式全量 Options 页
- 自测 / 带宽图高级交互

---

## 3. 技术选型

| 模块 | 选型 | 说明 |
|------|------|------|
| GUI | `gpui` + `gpui-component` + `gpui_platform` | 与 pass_smash 同栈 |
| HTTP | `hyper` + `tokio` + `tower` | 异步高性能，Range/流式友好 |
| 路由/中间件 | 自研薄层 | 避免过重框架，方便对齐 HFS 语义 |
| 序列化 | `serde` + `serde_json` | 配置与 VFS 存档 |
| 文件对话框 | `rfd` | 添加文件/目录 |
| 并发状态 | `parking_lot` + `Arc` | UI 与 server 共享 |
| 日志 | `tracing` | 内部诊断；业务日志单独通道给 UI |
| 压缩下载 | `zip`（P1） | archive 下载 |
| 剪贴板 | gpui / arboard（择一） | Copy URL |

依赖风格尽量贴近 `pass_smash`：

```toml
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit"] }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }
gpui-component-assets = { git = "https://github.com/longbridge/gpui-component" }
```

---

## 4. 架构设计

```
┌──────────────────────────────────────────────────────────┐
│                     GPUI Main Window                     │
│  Toolbar / URL / Graph / VFS Tree / Conns / Log / SBar   │
└───────────────┬──────────────────────▲───────────────────┘
                │ commands             │ UiEvent (poll)
                ▼                      │
┌──────────────────────────┐  ┌────────────────────────────┐
│     AppState (shared)    │  │   Event bus / mailbox      │
│  config, vfs, runtime    │◄─┤  log, conn, stats, status  │
└───────────────┬──────────┘  └────────────────────────────┘
                │
                ▼
┌──────────────────────────────────────────────────────────┐
│                 HTTP Server (tokio runtime)              │
│  accept → auth → resolve VFS path → list/download/upload │
└──────────────────────────────────────────────────────────┘
```

### 模块划分

```
src/
  main.rs           # 入口、panic log、开窗
  app.rs            # 根状态聚合（可选）
  config/           # 端口、限速、模式、持久化
  vfs/              # 虚拟文件系统模型与操作
  http/             # 请求处理、MIME、Range、目录页
  server/           # 生命周期：start/stop/stats
  ui/               # GPUI 视图组件
  util/             # 格式化、路径、URL、时间
templates/          # 内置 HTML 模板
docs/               # 规划与进度
```

### 关键约束

1. **UI 线程不阻塞**：文件 IO / HTTP accept 全在后台
2. **单一真相源**：`AppState` 持有 VFS 与 Config；Server 只读快照或锁
3. **事件驱动刷新**：server → mailbox → UI 50ms poll（参考 pass_smash）
4. **可测试核心**：VFS/HTTP 纯逻辑不依赖 GPUI

---

## 5. VFS 模型（对齐 HFS2 `Tfile`）

```text
VfsNode
  - id: NodeId
  - name: String                 # 展示名（可 rename）
  - kind: File | RealFolder | VirtualFolder | Link | Root
  - resource: Option<PathBuf>    # 真实路径 / URL
  - children: Vec<NodeId>        # 仅目录
  - parent: Option<NodeId>
  - flags: hidden, download_forbidden, hide_tree, dont_log, ...
  - comment / user_pass / masks  # P1
```

能力：

- 添加文件/文件夹到任意虚拟节点
- 新建空虚拟目录
- 删除、重命名、移动
- URL 路径解析：`/folder/file` → node
- 真实目录动态展开（real folder 浏览磁盘）
- 序列化/反序列化

---

## 6. HTTP 语义（P0）

| 请求 | 行为 |
|------|------|
| `GET /` | 列出 VFS 根 |
| `GET /path/` | 目录列表 HTML |
| `GET /path/file` | 下载文件，支持 `Range` |
| `HEAD` | 元数据 |
| `GET /~img` 等 | 内置资源（后续） |
| 未命中 | 404 页 |

响应头尽量兼容常见下载器：`Content-Type`、`Content-Length`、`Accept-Ranges`、`Content-Disposition`。

---

## 7. UI 实现策略

### 布局（近似 HFS2）

```
+------------------------------------------------------+
| [Menu] [Port:8080] [Easy] [Server ON/OFF]   status   |
| URL: http://127.0.0.1:8080/     [Open] [Copy]        |
+---------------------------+--------------------------+
| Bandwidth Graph (collapsible)                        |
+---------------------------+--------------------------+
| Virtual File System       | Connections              |
| (tree)                    | IP | File | Status | ... |
+---------------------------+--------------------------+
| Log (searchable)                                     |
+------------------------------------------------------+
| status bar: connections / bytes / uptime             |
+------------------------------------------------------+
```

### 组件映射

| HFS2 | gpui-component |
|------|----------------|
| Button / ToolButton | `Button` |
| Edit | `Input` + `InputState` |
| TreeView | 自绘树（div 缩进 + 展开）或 List |
| ListView | 表格行 `h_flex` |
| PopupMenu | `DropdownMenu` / 自绘 context menu |
| Options tabs | `Tab` / 多页 `v_flex` |
| Graph | 自绘 canvas/path 或 bar 近似 |

先追求**信息架构与操作路径一致**，像素级复刻不作为 P0 阻塞项。

---

## 8. 里程碑

| 里程碑 | 目标 | 完成标准 |
|--------|------|----------|
| M0 | 工程骨架 + 文档 | 可编译空窗，文档齐全 |
| M1 | VFS + 基础 HTTP | 可添加文件并浏览器下载 |
| M2 | 主界面三栏可用 | 树/连接/日志联动 |
| M3 | 启停/端口/URL/日志完善 | 日常分享可用 |
| M4 | 账号/上传/限速 | 权限场景可用 |
| M5 | 模板/Options/持久化 | 接近 HFS2 常用功能 |
| M6 | 抛光与打包 | release 安装/单文件分发 |

---

## 9. 开发约定

- Edition：2024（与 pass_smash 一致；若工具链不支持则退 2021）
- 中文注释可适度用于模块头；业务代码保持清晰命名
- 不引入与目标无关的大型框架
- 每个里程碑更新 `docs/PROGRESS.md`
- 核心逻辑优先单测（VFS 路径解析、Range 解析）
- Windows 为第一目标平台（HFS2 本就是 Windows 工具）

---

## 10. 风险与对策

| 风险 | 对策 |
|------|------|
| GPUI API 变动快 | 锁定与 pass_smash 相同 git 依赖策略；UI 薄封装 |
| HFS2 宏模板复杂 | P0 用静态/简单模板；P2 再做宏子集 |
| 大文件/多连接性能 | hyper 流式 + 独立 runtime；UI 降频刷新 |
| Tree/右键菜单组件不足 | 自绘交互，先功能后美观 |
| 端口 80 需管理员权限 | 默认 8080，UI 提示 |

---

## 11. 当前阶段目标（立即执行）

1. 建立仓库骨架与模块边界
2. 实现最小 VFS + HTTP 下载
3. 实现 GPUI 主窗壳（工具栏 + VFS 列表 + 日志 + 启停）
4. 跑通：添加文件 → Start → 浏览器下载

---

## 12. 非目标（明确不做/后置）

- 不兼容 HFS3 插件生态
- 不追求 Delphi 源码逐行移植
- 不在 P0 实现完整宏语言、tray、shell 集成
- 不做跨平台像素级系统托盘（后续按需）
