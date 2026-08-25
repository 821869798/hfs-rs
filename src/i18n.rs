//! Process-wide UI locale (Chinese / English / System Auto-detection).

use std::sync::atomic::{AtomicU8, Ordering};

use serde::{Deserialize, Serialize};

static LOCALE: AtomicU8 = AtomicU8::new(0); // 0 = System, 1 = Zh, 2 = En

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum Locale {
    #[default]
    System = 0,
    Zh = 1,
    En = 2,
}

#[cfg(windows)]
pub fn detect_system_is_zh() -> bool {
    if let Ok(lang) = std::env::var("LANG").or_else(|_| std::env::var("LC_ALL")) {
        if lang.to_ascii_lowercase().starts_with("zh") {
            return true;
        }
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetUserDefaultUILanguage() -> u16;
    }
    unsafe {
        let lcid = GetUserDefaultUILanguage();
        // Chinese language ID has low byte 0x04 (0x0804 = zh-CN, 0x0404 = zh-TW, etc.)
        (lcid & 0xFF) == 0x04
    }
}

#[cfg(not(windows))]
pub fn detect_system_is_zh() -> bool {
    if let Ok(lang) = std::env::var("LANG").or_else(|_| std::env::var("LC_ALL")) {
        lang.to_ascii_lowercase().starts_with("zh")
    } else {
        false
    }
}

impl Locale {
    pub fn resolve(self) -> Self {
        match self {
            Self::System => {
                if detect_system_is_zh() {
                    Self::Zh
                } else {
                    Self::En
                }
            }
            other => other,
        }
    }

    pub fn native_name(self) -> &'static str {
        match self {
            Self::System => "跟随系统",
            Self::Zh => "简体中文",
            Self::En => "English",
        }
    }

    pub fn toggle(self) -> Self {
        match self.resolve() {
            Self::Zh => Self::En,
            _ => Self::Zh,
        }
    }

    pub fn switch_label(self) -> &'static str {
        match self.resolve() {
            Self::Zh => "English",
            _ => "中文",
        }
    }
}

pub fn current() -> Locale {
    match LOCALE.load(Ordering::Relaxed) {
        1 => Locale::Zh,
        2 => Locale::En,
        _ => Locale::System.resolve(),
    }
}

pub fn set(locale: Locale) {
    LOCALE.store(locale as u8, Ordering::Relaxed);
}

/// UI string table.
#[derive(Debug, Clone, Copy)]
pub enum Msg {
    AppTitle,
    Menu,
    Port,
    EasyMode,
    ExpertMode,
    ServerOff,
    ServerOn,
    ServerStarting,
    ServerStopping,
    OpenInBrowser,
    CopyToClipboard,
    UrlPlaceholder,
    VirtualFileSystem,
    Log,
    Search,
    ClearLog,
    Connections,
    IpAddress,
    File,
    Status,
    Speed,
    TimeLeft,
    Progress,
    AddFiles,
    AddFolder,
    NewEmptyFolder,
    NewLink,
    Remove,
    Rename,
    CopyUrl,
    Properties,
    OpenItem,
    BrowseIt,
    Paste,
    Language,
    ShowBandwidthGraph,
    HideBandwidthGraph,
    BandwidthGraph,
    Options,
    OtherOptions,
    UserAccounts,
    SelfTest,
    AutoCopyUrl,
    AlwaysOnTop,
    SendHfsId,
    BrowseLocalhost,
    ComingSoon,
    ReadyHint,
    AddedItems,
    RemovedItem,
    RenamedItem,
    CreatedFolder,
    CopiedUrl,
    ClipboardError,
    OpenBrowserFailed,
    AddFailed,
    RemoveFailed,
    RenameFailed,
    CreateFolderFailed,
    NoSelection,
    RootCannotRemove,
    ListeningOn,
    OutBytes,
    InBytes,
    Mode,
    StatusBarBrand,
    ConnEmpty,
    VfsEmpty,
    KindRoot,
    KindVirtual,
    KindRealFolder,
    KindFile,
    KindLink,
    Downloading,
    Partial,
    Done,
    Aborted,
    GraphHint,
    PortLockedWhileRunning,
    RenamePrompt,
    LinkPrompt,
    PropertiesTitle,
    Resource,
    Comment,
    Close,
    Save,
    Cancel,
    ConfirmRename,
    ConfirmLink,
    MenuSelfTestOk,
    MenuAccountsSoon,
    MenuOptionsSoon,
    ToggleGraph,
    Expanded,
    Collapsed,
    Upload,
    ProtectUploads,
    NoAccounts,
    HtmlTemplate,
    ChangePort,
    DefaultSorting,
    // Detailed localization additions
    TabGeneral,
    TabServer,
    TabSecurity,
    TabAbout,
    SelectLanguageDesc,
    ThemeAppearance,
    ThemeAppearanceDesc,
    ThemeSystem,
    ThemeDark,
    ThemeLight,
    LangSystem,
    LangZh,
    LangEn,
    ModeDesc,
    AutoCopyUrlDesc,
    SendHfsIdDesc,
    BrowseLocalhostDesc,
    PortDesc,
    MaxConnections,
    MaxConnectionsDesc,
    MaxUploadSize,
    MaxUploadSizeDesc,
    BandwidthGraphDesc,
    ApplyServerSettings,
    UploadPolicy,
    UploadPolicyDesc,
    UploadDisabled,
    UploadPublic,
    UploadProtected,
    AccountsList,
    AddNewAccount,
    Username,
    Password,
    Add,
    AboutDesc,
    VersionInfo,
    CoreEngine,
    CoreEngineDesc,
    UiFramework,
    UiFrameworkDesc,
    Compatibility,
    CompatibilityDesc,
    License,
    LicenseDesc,
    FilterLogs,
    Clear,
    CopyAll,
    NoLogs,
    NoConnections,
    VfsToolbarAddFile,
    VfsToolbarAddFolder,
    VfsToolbarAddVirtual,
    VfsToolbarAddLink,
    VfsToolbarRename,
    VfsToolbarDelete,
    VfsToolbarProps,
    Online,
    Offline,
    Eta,
}

impl Msg {
    pub fn get(self, locale: Locale) -> &'static str {
        let loc = locale.resolve();
        match (self, loc) {
            (Self::AppTitle, Locale::Zh) => "HFS-RS ~ HTTP 文件服务器",
            (Self::AppTitle, _) => "HFS-RS ~ HTTP File Server",
            (Self::Menu, Locale::Zh) => "菜单",
            (Self::Menu, _) => "Menu",
            (Self::Port, Locale::Zh) => "端口",
            (Self::Port, _) => "Port",
            (Self::EasyMode, Locale::Zh) => "简易模式",
            (Self::EasyMode, _) => "Easy mode",
            (Self::ExpertMode, Locale::Zh) => "专家模式",
            (Self::ExpertMode, _) => "Expert mode",
            (Self::ServerOff, Locale::Zh) => "服务器已停止",
            (Self::ServerOff, _) => "Server is Stopped",
            (Self::ServerOn, Locale::Zh) => "服务器运行中",
            (Self::ServerOn, _) => "Server is Running",
            (Self::ServerStarting, Locale::Zh) => "正在启动…",
            (Self::ServerStarting, _) => "Starting…",
            (Self::ServerStopping, Locale::Zh) => "正在停止…",
            (Self::ServerStopping, _) => "Stopping…",
            (Self::OpenInBrowser, Locale::Zh) => "浏览器打开",
            (Self::OpenInBrowser, _) => "Open in browser",
            (Self::CopyToClipboard, Locale::Zh) => "复制链接",
            (Self::CopyToClipboard, _) => "Copy link",
            (Self::UrlPlaceholder, Locale::Zh) => "http://localhost:8080/",
            (Self::UrlPlaceholder, _) => "http://localhost:8080/",
            (Self::VirtualFileSystem, Locale::Zh) => "虚拟文件系统",
            (Self::VirtualFileSystem, _) => "Virtual File System",
            (Self::Log, Locale::Zh) => "日志记录",
            (Self::Log, _) => "Log",
            (Self::Search, Locale::Zh) => "搜索…",
            (Self::Search, _) => "Search…",
            (Self::ClearLog, Locale::Zh) => "清空日志",
            (Self::ClearLog, _) => "Clear log",
            (Self::Connections, Locale::Zh) => "连接",
            (Self::Connections, _) => "Connections",
            (Self::IpAddress, Locale::Zh) => "IP 地址",
            (Self::IpAddress, _) => "IP address",
            (Self::File, Locale::Zh) => "文件",
            (Self::File, _) => "File",
            (Self::Status, Locale::Zh) => "状态",
            (Self::Status, _) => "Status",
            (Self::Speed, Locale::Zh) => "速度",
            (Self::Speed, _) => "Speed",
            (Self::TimeLeft, Locale::Zh) => "剩余时间",
            (Self::TimeLeft, _) => "Time left",
            (Self::Progress, Locale::Zh) => "进度",
            (Self::Progress, _) => "Progress",
            (Self::AddFiles, Locale::Zh) => "添加文件…",
            (Self::AddFiles, _) => "Add files...",
            (Self::AddFolder, Locale::Zh) => "添加文件夹…",
            (Self::AddFolder, _) => "Add folder...",
            (Self::NewEmptyFolder, Locale::Zh) => "新建虚拟文件夹",
            (Self::NewEmptyFolder, _) => "New virtual folder",
            (Self::NewLink, Locale::Zh) => "新建链接",
            (Self::NewLink, _) => "New link",
            (Self::Remove, Locale::Zh) => "移除",
            (Self::Remove, _) => "Remove",
            (Self::Rename, Locale::Zh) => "重命名",
            (Self::Rename, _) => "Rename",
            (Self::CopyUrl, Locale::Zh) => "复制 URL 地址",
            (Self::CopyUrl, _) => "Copy URL address",
            (Self::Properties, Locale::Zh) => "属性…",
            (Self::Properties, _) => "Properties...",
            (Self::OpenItem, Locale::Zh) => "在资源管理器中打开",
            (Self::OpenItem, _) => "Open in Explorer",
            (Self::BrowseIt, Locale::Zh) => "浏览器浏览",
            (Self::BrowseIt, _) => "Browse it",
            (Self::Paste, Locale::Zh) => "粘贴",
            (Self::Paste, _) => "Paste",
            (Self::Language, Locale::Zh) => "语言",
            (Self::Language, _) => "Language",
            (Self::ShowBandwidthGraph, Locale::Zh) => "显示带宽图",
            (Self::ShowBandwidthGraph, _) => "Show bandwidth graph",
            (Self::HideBandwidthGraph, Locale::Zh) => "隐藏带宽图",
            (Self::HideBandwidthGraph, _) => "Hide bandwidth graph",
            (Self::BandwidthGraph, Locale::Zh) => "带宽图",
            (Self::BandwidthGraph, _) => "Bandwidth graph",
            (Self::Options, Locale::Zh) => "选项",
            (Self::Options, _) => "Options",
            (Self::OtherOptions, Locale::Zh) => "选项",
            (Self::OtherOptions, _) => "Options",
            (Self::UserAccounts, Locale::Zh) => "用户账号…",
            (Self::UserAccounts, _) => "User accounts...",
            (Self::SelfTest, Locale::Zh) => "自检",
            (Self::SelfTest, _) => "Self Test",
            (Self::AutoCopyUrl, Locale::Zh) => "添加时自动复制 URL",
            (Self::AutoCopyUrl, _) => "Auto-copy URL on addition",
            (Self::AlwaysOnTop, Locale::Zh) => "窗口置顶",
            (Self::AlwaysOnTop, _) => "Always on top",
            (Self::SendHfsId, Locale::Zh) => "发送 HFS 标识",
            (Self::SendHfsId, _) => "Send HFS identifier",
            (Self::BrowseLocalhost, Locale::Zh) => "使用 localhost 浏览",
            (Self::BrowseLocalhost, _) => "Browse using localhost",
            (Self::ComingSoon, Locale::Zh) => "该功能即将到来",
            (Self::ComingSoon, _) => "Coming in a later milestone",
            (Self::ReadyHint, Locale::Zh) => "欢迎使用 HFS-RS — 添加文件后启动服务器。",
            (Self::ReadyHint, _) => "Welcome to HFS-RS — add files, then start the server.",
            (Self::AddedItems, Locale::Zh) => "已添加 {n} 项",
            (Self::AddedItems, _) => "Added {n} item(s)",
            (Self::RemovedItem, Locale::Zh) => "已移除项",
            (Self::RemovedItem, _) => "Removed item",
            (Self::RenamedItem, Locale::Zh) => "已重命名",
            (Self::RenamedItem, _) => "Renamed",
            (Self::CreatedFolder, Locale::Zh) => "已创建文件夹",
            (Self::CreatedFolder, _) => "Created folder",
            (Self::CopiedUrl, Locale::Zh) => "已复制 URL 到剪贴板",
            (Self::CopiedUrl, _) => "Copied URL to clipboard",
            (Self::ClipboardError, Locale::Zh) => "无法访问剪贴板",
            (Self::ClipboardError, _) => "Clipboard access error",
            (Self::OpenBrowserFailed, Locale::Zh) => "打开浏览器失败",
            (Self::OpenBrowserFailed, _) => "Failed to open browser",
            (Self::AddFailed, Locale::Zh) => "添加失败：{err}",
            (Self::AddFailed, _) => "Failed to add: {err}",
            (Self::RemoveFailed, Locale::Zh) => "移除失败：{err}",
            (Self::RemoveFailed, _) => "Failed to remove: {err}",
            (Self::RenameFailed, Locale::Zh) => "重命名失败：{err}",
            (Self::RenameFailed, _) => "Failed to rename: {err}",
            (Self::CreateFolderFailed, Locale::Zh) => "创建文件夹失败：{err}",
            (Self::CreateFolderFailed, _) => "Failed to create folder: {err}",
            (Self::NoSelection, Locale::Zh) => "请先选择一个节点",
            (Self::NoSelection, _) => "No item selected",
            (Self::RootCannotRemove, Locale::Zh) => "根节点不可移除或重命名",
            (Self::RootCannotRemove, _) => "Root folder cannot be removed or renamed",
            (Self::ListeningOn, Locale::Zh) => "正在监听",
            (Self::ListeningOn, _) => "Listening on",
            (Self::OutBytes, Locale::Zh) => "出站",
            (Self::OutBytes, _) => "Out",
            (Self::InBytes, Locale::Zh) => "入站",
            (Self::InBytes, _) => "In",
            (Self::Mode, Locale::Zh) => "模式",
            (Self::Mode, _) => "Mode",
            (Self::StatusBarBrand, Locale::Zh) => "HFS-RS v0.1.0 (Rust & GPUI)",
            (Self::StatusBarBrand, _) => "HFS-RS v0.1.0 (Rust & GPUI)",
            (Self::ConnEmpty, Locale::Zh) => "无活动连接",
            (Self::ConnEmpty, _) => "No active connections",
            (Self::VfsEmpty, Locale::Zh) => "从磁盘拖放文件到此处，或右键添加。",
            (Self::VfsEmpty, _) => "Drop files here from disk, or right-click to add.",
            (Self::KindRoot, Locale::Zh) => "根目录",
            (Self::KindRoot, _) => "Root",
            (Self::KindVirtual, Locale::Zh) => "虚拟文件夹",
            (Self::KindVirtual, _) => "Virtual folder",
            (Self::KindRealFolder, Locale::Zh) => "真实文件夹",
            (Self::KindRealFolder, _) => "Real folder",
            (Self::KindFile, Locale::Zh) => "文件",
            (Self::KindFile, _) => "File",
            (Self::KindLink, Locale::Zh) => "链接",
            (Self::KindLink, _) => "Link",
            (Self::Downloading, Locale::Zh) => "下载中",
            (Self::Downloading, _) => "Downloading",
            (Self::Partial, Locale::Zh) => "断点续传",
            (Self::Partial, _) => "Partial",
            (Self::Done, Locale::Zh) => "已完成",
            (Self::Done, _) => "Done",
            (Self::Aborted, Locale::Zh) => "已中断",
            (Self::Aborted, _) => "Aborted",
            (Self::GraphHint, Locale::Zh) => "红色：出站（下载）  黄色：入站（上传）",
            (Self::GraphHint, _) => "Red: Out (downloads)  Yellow: In (uploads)",
            (Self::PortLockedWhileRunning, Locale::Zh) => {
                "服务器运行期间端口已锁定，请先停止服务器。"
            }
            (Self::PortLockedWhileRunning, _) => {
                "Port cannot be changed while the server is running. Stop it first."
            }
            (Self::RenamePrompt, Locale::Zh) => "请输入新名称：",
            (Self::RenamePrompt, _) => "Enter new name:",
            (Self::LinkPrompt, Locale::Zh) => "请输入目标 URL：",
            (Self::LinkPrompt, _) => "Enter target URL:",
            (Self::PropertiesTitle, Locale::Zh) => "文件属性",
            (Self::PropertiesTitle, _) => "Properties",
            (Self::Resource, Locale::Zh) => "真实路径：",
            (Self::Resource, _) => "Resource path:",
            (Self::Comment, Locale::Zh) => "备注：",
            (Self::Comment, _) => "Comment:",
            (Self::Close, Locale::Zh) => "关闭",
            (Self::Close, _) => "Close",
            (Self::Save, Locale::Zh) => "保存",
            (Self::Save, _) => "Save",
            (Self::Cancel, Locale::Zh) => "取消",
            (Self::Cancel, _) => "Cancel",
            (Self::ConfirmRename, Locale::Zh) => "确定",
            (Self::ConfirmRename, _) => "OK",
            (Self::ConfirmLink, Locale::Zh) => "添加链接",
            (Self::ConfirmLink, _) => "Add Link",
            (Self::MenuSelfTestOk, Locale::Zh) => "自检通过：服务器就绪。",
            (Self::MenuSelfTestOk, _) => "Self test passed: server is ready.",
            (Self::MenuAccountsSoon, Locale::Zh) => "账号管理界面",
            (Self::MenuAccountsSoon, _) => "User accounts dialog",
            (Self::MenuOptionsSoon, Locale::Zh) => "选项配置界面",
            (Self::MenuOptionsSoon, _) => "Options dialog",
            (Self::ToggleGraph, Locale::Zh) => "切换流量图",
            (Self::ToggleGraph, _) => "Toggle graph",
            (Self::Expanded, Locale::Zh) => "已展开",
            (Self::Expanded, _) => "Expanded",
            (Self::Collapsed, Locale::Zh) => "已折叠",
            (Self::Collapsed, _) => "Collapsed",
            (Self::Upload, Locale::Zh) => "文件上传",
            (Self::Upload, _) => "File Upload",
            (Self::ProtectUploads, Locale::Zh) => "上传需账号密码保护",
            (Self::ProtectUploads, _) => "Protect uploads with password",
            (Self::NoAccounts, Locale::Zh) => "当前无配置用户账号（点击下方添加）",
            (Self::NoAccounts, _) => "No accounts configured yet (add one below)",
            (Self::HtmlTemplate, Locale::Zh) => "HTML 模板",
            (Self::HtmlTemplate, _) => "HTML Template",
            (Self::ChangePort, Locale::Zh) => "修改端口",
            (Self::ChangePort, _) => "Change Port",
            (Self::DefaultSorting, Locale::Zh) => "默认排序",
            (Self::DefaultSorting, _) => "Default Sorting",

            // Detailed localization additions
            (Self::TabGeneral, Locale::Zh) => "常规设置",
            (Self::TabGeneral, _) => "General",
            (Self::TabServer, Locale::Zh) => "服务与网络",
            (Self::TabServer, _) => "Server & Network",
            (Self::TabSecurity, Locale::Zh) => "上传与安全",
            (Self::TabSecurity, _) => "Uploads & Security",
            (Self::TabAbout, Locale::Zh) => "关于",
            (Self::TabAbout, _) => "About",

            (Self::SelectLanguageDesc, Locale::Zh) => "选择界面显示语言",
            (Self::SelectLanguageDesc, _) => "Select UI display language",
            (Self::ThemeAppearance, Locale::Zh) => "主题外观",
            (Self::ThemeAppearance, _) => "Theme Appearance",
            (Self::ThemeAppearanceDesc, Locale::Zh) => "在深色、浅色或跟随系统主题间切换",
            (Self::ThemeAppearanceDesc, _) => "Switch between Dark, Light, and System theme",
            (Self::ThemeSystem, Locale::Zh) => "🖥️ 跟随系统",
            (Self::ThemeSystem, _) => "🖥️ System Default",
            (Self::ThemeDark, Locale::Zh) => "🌙 深色模式",
            (Self::ThemeDark, _) => "🌙 Dark Mode",
            (Self::ThemeLight, Locale::Zh) => "☀️ 浅色模式",
            (Self::ThemeLight, _) => "☀️ Light Mode",
            (Self::LangSystem, Locale::Zh) => "🖥️ 跟随系统",
            (Self::LangSystem, _) => "🖥️ System Default",
            (Self::LangZh, Locale::Zh) => "🇨🇳 简体中文",
            (Self::LangZh, _) => "🇨🇳 Chinese (Simplified)",
            (Self::LangEn, Locale::Zh) => "🇺🇸 English",
            (Self::LangEn, _) => "🇺🇸 English",

            (Self::ModeDesc, Locale::Zh) => "简易模式隐藏技术选项，专家模式展示全部功能",
            (Self::ModeDesc, _) => "Easy mode hides technical options, Expert mode shows all",
            (Self::AutoCopyUrlDesc, Locale::Zh) => "添加新文件或文件夹时自动将 URL 复制到剪贴板",
            (Self::AutoCopyUrlDesc, _) => {
                "Automatically copy file URL when adding new files/folders"
            }
            (Self::SendHfsIdDesc, Locale::Zh) => "在 HTTP 响应头中包含 HFS 服务标识信息",
            (Self::SendHfsIdDesc, _) => "Send HFS server identifier in HTTP response headers",
            (Self::BrowseLocalhostDesc, Locale::Zh) => {
                "在浏览器中打开时优先使用 127.0.0.1 本地回环地址"
            }
            (Self::BrowseLocalhostDesc, _) => {
                "Use 127.0.0.1 instead of public IP when opening browser"
            }

            (Self::PortDesc, Locale::Zh) => "HTTP 服务监听端口（如 80、8080）",
            (Self::PortDesc, _) => "HTTP port to listen on (e.g. 80, 8080)",
            (Self::MaxConnections, Locale::Zh) => "最大同时连接数",
            (Self::MaxConnections, _) => "Max Simultaneous Connections",
            (Self::MaxConnectionsDesc, Locale::Zh) => "限制总活跃并发连接数（留空表示无限制）",
            (Self::MaxConnectionsDesc, _) => {
                "Limit total active HTTP connections (empty for unlimited)"
            }
            (Self::MaxUploadSize, Locale::Zh) => "单文件最大上传限制 (MB)",
            (Self::MaxUploadSize, _) => "Max Upload Size (MB)",
            (Self::MaxUploadSizeDesc, Locale::Zh) => "单次文件上传允许的最大体积（兆字节）",
            (Self::MaxUploadSizeDesc, _) => "Maximum single upload size limit in Megabytes",
            (Self::BandwidthGraphDesc, Locale::Zh) => "在主界面实时显示出站/入站带宽波形图",
            (Self::BandwidthGraphDesc, _) => {
                "Display real-time Out/In bandwidth waveform on main interface"
            }
            (Self::ApplyServerSettings, Locale::Zh) => "应用服务设置",
            (Self::ApplyServerSettings, _) => "Apply Server Settings",

            (Self::UploadPolicy, Locale::Zh) => "上传策略",
            (Self::UploadPolicy, _) => "Upload Policy",
            (Self::UploadPolicyDesc, Locale::Zh) => "配置文件上传访问权限与认证保护",
            (Self::UploadPolicyDesc, _) => "Select HTTP upload access and authentication policy",
            (Self::UploadDisabled, Locale::Zh) => "⛔ 禁止上传",
            (Self::UploadDisabled, _) => "⛔ Disabled",
            (Self::UploadPublic, Locale::Zh) => "🔓 允许公开上传",
            (Self::UploadPublic, _) => "🔓 Public (Allow All)",
            (Self::UploadProtected, Locale::Zh) => "🔐 需账号认证",
            (Self::UploadProtected, _) => "🔐 Protected (Require Auth)",
            (Self::AccountsList, Locale::Zh) => "已配置账号列表",
            (Self::AccountsList, _) => "Configured Accounts",
            (Self::AddNewAccount, Locale::Zh) => "添加新账号",
            (Self::AddNewAccount, _) => "Add New Account",
            (Self::Username, Locale::Zh) => "用户名",
            (Self::Username, _) => "Username",
            (Self::Password, Locale::Zh) => "密码",
            (Self::Password, _) => "Password",
            (Self::Add, Locale::Zh) => "添加",
            (Self::Add, _) => "Add",

            (Self::AboutDesc, Locale::Zh) => {
                "基于 Rust 与 GPUI 重构的高性能 HTTP 文件服务器（兼容经典 HFS 2）"
            }
            (Self::AboutDesc, _) => {
                "High-Performance HTTP File Server in Rust with GPUI (HFS 2 Compatible)"
            }
            (Self::VersionInfo, Locale::Zh) => "版本 0.1.0 (2026 版)",
            (Self::VersionInfo, _) => "Version 0.1.0 (2026 Edition)",
            (Self::CoreEngine, Locale::Zh) => "核心引擎",
            (Self::CoreEngine, _) => "Core Engine",
            (Self::CoreEngineDesc, Locale::Zh) => {
                "Tokio 多线程异步 HTTP/1.1，支持 Range 断点续传与流式分块上传"
            }
            (Self::CoreEngineDesc, _) => "Tokio multi-threaded HTTP/1.1 with Range & Multipart",
            (Self::UiFramework, Locale::Zh) => "界面引擎",
            (Self::UiFrameworkDesc, Locale::Zh) => "Zed GPUI（纯原生 GPU 硬件加速桌面渲染框架）",
            (Self::UiFramework, _) => "UI Framework",
            (Self::UiFrameworkDesc, _) => "Zed GPUI (Pure High-Performance Desktop GPU Engine)",
            (Self::Compatibility, Locale::Zh) => "兼容特性",
            (Self::CompatibilityDesc, Locale::Zh) => {
                "HFS 2 虚拟文件系统 (VFS)、用户权限认证体系与模板机制"
            }
            (Self::Compatibility, _) => "Compatibility",
            (Self::CompatibilityDesc, _) => "HFS 2 Virtual File System, Accounts & Templates",
            (Self::License, Locale::Zh) => "开源协议",
            (Self::LicenseDesc, Locale::Zh) => "MIT 开源协议",
            (Self::License, _) => "License",
            (Self::LicenseDesc, _) => "MIT Open Source License",

            (Self::FilterLogs, Locale::Zh) => "过滤日志…",
            (Self::FilterLogs, _) => "Filter logs…",
            (Self::Clear, Locale::Zh) => "清空",
            (Self::Clear, _) => "Clear",
            (Self::CopyAll, Locale::Zh) => "复制全部",
            (Self::CopyAll, _) => "Copy All",
            (Self::NoLogs, Locale::Zh) => "暂无日志记录",
            (Self::NoLogs, _) => "No logs recorded",
            (Self::NoConnections, Locale::Zh) => "当前无活跃连接",
            (Self::NoConnections, _) => "No active connections",
            (Self::VfsToolbarAddFile, Locale::Zh) => "+ 文件",
            (Self::VfsToolbarAddFile, _) => "+ File",
            (Self::VfsToolbarAddFolder, Locale::Zh) => "+ 文件夹",
            (Self::VfsToolbarAddFolder, _) => "+ Folder",
            (Self::VfsToolbarAddVirtual, Locale::Zh) => "+ 虚拟",
            (Self::VfsToolbarAddVirtual, _) => "+ Virtual",
            (Self::VfsToolbarAddLink, Locale::Zh) => "+ 链接",
            (Self::VfsToolbarAddLink, _) => "+ Link",
            (Self::VfsToolbarRename, Locale::Zh) => "重命名",
            (Self::VfsToolbarRename, _) => "Rename",
            (Self::VfsToolbarDelete, Locale::Zh) => "删除",
            (Self::VfsToolbarDelete, _) => "Delete",
            (Self::VfsToolbarProps, Locale::Zh) => "属性",
            (Self::VfsToolbarProps, _) => "Properties",
            (Self::Online, Locale::Zh) => "运行中",
            (Self::Online, _) => "ONLINE",
            (Self::Offline, Locale::Zh) => "已停止",
            (Self::Offline, _) => "OFFLINE",
            (Self::Eta, Locale::Zh) => "剩余",
            (Self::Eta, _) => "ETA",
        }
    }
}

pub fn tr(msg: Msg) -> &'static str {
    msg.get(current())
}

pub fn format_err(msg: Msg, err: impl std::fmt::Display) -> String {
    let raw = tr(msg);
    raw.replace("{err}", &err.to_string())
}

pub fn format_n(msg: Msg, n: usize) -> String {
    let raw = tr(msg);
    raw.replace("{n}", &n.to_string())
}

pub fn format_listening(addr: &str) -> String {
    let raw = tr(Msg::ListeningOn);
    format!("{raw} http://{addr}/")
}

pub fn format_copied_url(url: &str) -> String {
    let raw = tr(Msg::CopiedUrl);
    format!("{raw}: {url}")
}

pub fn format_added(n: usize) -> String {
    format_n(Msg::AddedItems, n)
}
