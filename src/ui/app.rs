//! Main HFS-RS Application Window — HFS2 layout and architecture.

use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Context, Entity, ExternalPaths, FontWeight, IntoElement, MouseButton, Render, SharedString,
    Window, div, prelude::*, px,
};

use crate::i18n::{self, Locale, Msg};
use crate::server::{AppEvent, AppState, ConnInfo, LogLevel, ServerStatus};
use crate::ui::components::{CustomButton, render_badge};
use crate::ui::conn_view::{ConnViewProps, render_conn_pane};
use crate::ui::dialogs::{DialogKind, render_dialog_overlay};
use crate::ui::graph_view::{GraphPoint, GraphViewProps, render_graph_pane};
use crate::ui::log_view::{LogEntry, LogViewProps, render_log_pane};
use crate::ui::settings_view::{
    SettingsDropdownKind, SettingsTab, SettingsViewProps, render_settings_overlay,
};
use crate::ui::text_input::TextInput;
use crate::ui::theme::{Theme, ThemeMode};
use crate::ui::vfs_view::{VfsViewProps, render_vfs_context_menu, render_vfs_pane};
use crate::util::{format_bytes, format_speed};
use crate::vfs::NodeId;

#[derive(Clone, Copy, Debug, PartialEq)]
enum ResizingSplitter {
    LeftPanel { start_x: f32, initial_width: f32 },
    BottomPanel { start_y: f32, initial_height: f32 },
}

pub struct HfsApp {
    state: Arc<AppState>,
    theme: Theme,
    theme_mode: ThemeMode,
    locale: Locale,
    expert_mode: bool,
    show_graph: bool,
    selected: Option<NodeId>,
    expanded: HashSet<u64>,
    connections: Vec<ConnInfo>,
    logs: Vec<LogEntry>,
    status_text: SharedString,
    server_status: ServerStatus,
    bytes_out: u64,
    bytes_in: u64,
    conn_count: usize,
    out_bps: f64,
    in_bps: f64,
    graph_history: VecDeque<GraphPoint>,
    search_input: Entity<TextInput>,
    url_text: String,
    context_menu_open: bool,
    vfs_add_menu_open: bool,
    resizing_splitter: Option<ResizingSplitter>,
    dialog: DialogKind,
    pending_paths: Option<(Vec<PathBuf>, bool)>,
    left_panel_width: f32,
    bottom_panel_height: f32,
    settings_open: bool,
    settings_tab: SettingsTab,
    settings_open_dropdown: Option<SettingsDropdownKind>,
    settings_port_input: Entity<TextInput>,
    settings_max_conn_input: Entity<TextInput>,
    settings_upload_max_input: Entity<TextInput>,
    settings_new_user_input: Entity<TextInput>,
    settings_new_pass_input: Entity<TextInput>,
}

impl HfsApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = AppState::new();
        let cfg = state.config.read().clone();
        i18n::set(cfg.locale);

        let search_input = cx.new(|cx| TextInput::new(i18n::tr(Msg::Search), cx));

        let bus = Arc::clone(&state.bus);
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
                let mut events = bus.drain();
                if events.is_empty() {
                    continue;
                }
                if events.len() > 64 {
                    let overflow = events.len() - 64;
                    events.drain(0..overflow);
                }
                let res = this.update(cx, |this, cx| {
                    for ev in events {
                        this.handle_event(ev, cx);
                    }
                    cx.notify();
                });
                if res.is_err() {
                    break;
                }
            }
        })
        .detach();

        let mut expanded = HashSet::new();
        expanded.insert(state.vfs.read().root_id().0);

        let locale = cfg.locale;
        i18n::set(locale);

        let now_time = chrono::Local::now().format("%H:%M:%S").to_string();
        let initial_logs = vec![LogEntry {
            level: LogLevel::Info,
            time: now_time.into(),
            text: i18n::tr(Msg::ReadyHint).into(),
        }];

        let theme_mode = cfg.theme_mode;
        let theme = Theme::for_mode(theme_mode);

        let settings_port_input = cx.new(|cx| TextInput::new("8080", cx));
        let settings_max_conn_input = cx.new(|cx| TextInput::new("", cx));
        let settings_upload_max_input = cx.new(|cx| TextInput::new("512", cx));
        let settings_new_user_input = cx.new(|cx| TextInput::new("", cx));
        let settings_new_pass_input = cx.new(|cx| TextInput::new("", cx));

        Self {
            url_text: cfg.public_base_url(),
            expert_mode: cfg.expert_mode,
            locale,
            show_graph: cfg.show_bandwidth_graph,
            state,
            theme,
            theme_mode,
            selected: None,
            expanded,
            connections: Vec::new(),
            logs: initial_logs,
            status_text: i18n::tr(Msg::ServerOff).into(),
            server_status: ServerStatus::Stopped,
            bytes_out: 0,
            bytes_in: 0,
            conn_count: 0,
            out_bps: 0.0,
            in_bps: 0.0,
            graph_history: VecDeque::with_capacity(60),
            search_input,
            context_menu_open: false,
            vfs_add_menu_open: false,
            resizing_splitter: None,
            dialog: DialogKind::None,
            pending_paths: None,
            left_panel_width: 320.0,
            bottom_panel_height: 180.0,
            settings_open: false,
            settings_tab: SettingsTab::General,
            settings_open_dropdown: None,
            settings_port_input,
            settings_max_conn_input,
            settings_upload_max_input,
            settings_new_user_input,
            settings_new_pass_input,
        }
    }

    fn handle_event(&mut self, ev: AppEvent, cx: &mut Context<Self>) {
        match ev {
            AppEvent::Log { level, text } => {
                let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
                self.logs.push(LogEntry {
                    level,
                    time: time_str.into(),
                    text: text.into(),
                });
                if self.logs.len() > 1000 {
                    let overflow = self.logs.len() - 1000;
                    self.logs.drain(0..overflow);
                }
            }
            AppEvent::ServerStatus(status) => {
                self.server_status = status;
                self.status_text = match status {
                    ServerStatus::Stopped => i18n::tr(Msg::ServerOff),
                    ServerStatus::Starting => i18n::tr(Msg::ServerStarting),
                    ServerStatus::Running => i18n::tr(Msg::ServerOn),
                    ServerStatus::Stopping => i18n::tr(Msg::ServerStopping),
                }
                .into();
            }
            AppEvent::ServerStarted { addr } => {
                self.url_text = self.state.config.read().public_base_url();
                self.status_text = i18n::format_listening(&addr).into();
            }
            AppEvent::ServerStopped => {
                self.connections.clear();
                self.conn_count = 0;
            }
            AppEvent::ConnectionUpsert(info) => {
                if let Some(slot) = self.connections.iter_mut().find(|c| c.id == info.id) {
                    *slot = info;
                } else {
                    self.connections.push(info);
                }
            }
            AppEvent::ConnectionRemoved { id } => {
                self.connections.retain(|c| c.id != id);
            }
            AppEvent::Stats {
                bytes_out,
                bytes_in,
                connections,
            } => {
                self.bytes_out = bytes_out;
                self.bytes_in = bytes_in;
                self.conn_count = connections;
                let mut list: Vec<_> = self
                    .state
                    .server
                    .connections
                    .read()
                    .values()
                    .cloned()
                    .collect();
                list.sort_by(|a, b| a.peer.cmp(&b.peer).then(a.file.cmp(&b.file)));
                self.connections = list;
            }
            AppEvent::BandwidthSample { out_bps, in_bps } => {
                self.out_bps = out_bps;
                self.in_bps = in_bps;
                self.graph_history.push_back(GraphPoint { out_bps, in_bps });
                while self.graph_history.len() > 60 {
                    self.graph_history.pop_front();
                }
            }
            AppEvent::PathsPicked { paths, as_folder } => {
                self.add_paths(paths, as_folder, cx);
            }
        }
    }

    fn add_log(&self, level: LogLevel, text: impl Into<String>) {
        self.state.bus.log(level, text);
    }

    fn save_config(&self) {
        let snapshot = {
            let mut cfg = self.state.config.write();
            cfg.expert_mode = self.expert_mode;
            cfg.locale = self.locale;
            cfg.theme_mode = self.theme_mode;
            cfg.show_bandwidth_graph = self.show_graph;
            cfg.clone()
        };
        std::thread::Builder::new()
            .name("hfs-save-cfg".into())
            .spawn(move || {
                let _ = snapshot.save();
            })
            .ok();
    }

    fn server_running(&self) -> bool {
        matches!(
            self.server_status,
            ServerStatus::Running | ServerStatus::Starting
        )
    }

    fn toggle_server(&mut self, cx: &mut Context<Self>) {
        match self.server_status {
            ServerStatus::Stopped => {
                self.state.start_server();
            }
            ServerStatus::Running | ServerStatus::Starting => {
                self.state.stop_server();
            }
            ServerStatus::Stopping => {}
        }
        cx.notify();
    }

    fn toggle_mode(&mut self, cx: &mut Context<Self>) {
        self.expert_mode = !self.expert_mode;
        self.save_config();
        self.add_log(
            LogLevel::Info,
            if self.expert_mode {
                i18n::tr(Msg::ExpertMode)
            } else {
                i18n::tr(Msg::EasyMode)
            },
        );
        cx.notify();
    }

    #[allow(dead_code)]
    fn toggle_locale(&mut self, cx: &mut Context<Self>) {
        self.locale = self.locale.toggle();
        i18n::set(self.locale);
        self.search_input.update(cx, |input, cx| {
            input.set_placeholder(i18n::tr(Msg::Search), cx);
        });
        if !self.server_running() {
            self.status_text = i18n::tr(Msg::ServerOff).into();
        }
        self.save_config();
        cx.notify();
    }

    #[allow(dead_code)]
    fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        self.theme_mode = match self.theme_mode {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::System => {
                if crate::ui::theme::detect_system_is_dark() {
                    ThemeMode::Light
                } else {
                    ThemeMode::Dark
                }
            }
        };
        self.theme = Theme::for_mode(self.theme_mode);
        self.save_config();
        cx.notify();
    }

    fn toggle_graph(&mut self, cx: &mut Context<Self>) {
        self.show_graph = !self.show_graph;
        self.save_config();
        cx.notify();
    }

    fn target_parent(&self) -> NodeId {
        let vfs = self.state.vfs.read();
        if let Some(id) = self.selected {
            if let Some(node) = vfs.get(id) {
                if node.is_folder() {
                    return id;
                }
                if let Some(p) = node.parent {
                    return p;
                }
            }
        }
        vfs.root_id()
    }

    fn selected_url(&self) -> String {
        let base = self.state.config.read().public_base_url();
        let vfs = self.state.vfs.read();
        let path = match self.selected {
            Some(id) => vfs.url_path(id),
            None => "/".to_string(),
        };
        if base.ends_with('/') && path.starts_with('/') {
            format!("{}{}", base.trim_end_matches('/'), path)
        } else {
            format!("{base}{path}")
        }
    }

    fn copy_selected_url(&mut self, _cx: &mut Context<Self>) {
        let url = self.selected_url();
        match arboard::Clipboard::new().and_then(|mut c| c.set_text(url.clone())) {
            Ok(()) => self.add_log(LogLevel::Info, i18n::format_copied_url(&url)),
            Err(err) => self.add_log(LogLevel::Error, i18n::format_err(Msg::ClipboardError, err)),
        }
    }

    fn open_in_browser(&mut self, _cx: &mut Context<Self>) {
        let url = self.selected_url();
        if let Err(err) = open::that(&url) {
            self.add_log(
                LogLevel::Error,
                i18n::format_err(Msg::OpenBrowserFailed, err),
            );
        }
    }

    fn open_local_item(&mut self, _cx: &mut Context<Self>) {
        let Some(id) = self.selected else { return };
        let path = self
            .state
            .vfs
            .read()
            .get(id)
            .and_then(|n| n.resource.clone());
        if let Some(path) = path {
            let _ = open::that(path);
        }
    }

    fn add_files(&mut self, _cx: &mut Context<Self>) {
        let title = i18n::tr(Msg::AddFiles).to_string();
        let bus = Arc::clone(&self.state.bus);
        std::thread::Builder::new()
            .name("hfs-pick-files".into())
            .spawn(move || {
                let files = rfd::FileDialog::new().set_title(title).pick_files();
                if let Some(files) = files {
                    bus.push(AppEvent::PathsPicked {
                        paths: files,
                        as_folder: false,
                    });
                }
            })
            .ok();
    }

    fn add_folder(&mut self, _cx: &mut Context<Self>) {
        let title = i18n::tr(Msg::AddFolder).to_string();
        let bus = Arc::clone(&self.state.bus);
        std::thread::Builder::new()
            .name("hfs-pick-folder".into())
            .spawn(move || {
                let folder = rfd::FileDialog::new().set_title(title).pick_folder();
                if let Some(folder) = folder {
                    bus.push(AppEvent::PathsPicked {
                        paths: vec![folder],
                        as_folder: true,
                    });
                }
            })
            .ok();
    }

    fn add_paths(&mut self, paths: Vec<PathBuf>, as_folder: bool, cx: &mut Context<Self>) {
        let parent = self.target_parent();
        self.expanded.insert(parent.0);
        let mut added = 0usize;
        {
            let mut vfs = self.state.vfs.write();
            for path in paths {
                let result = if as_folder || path.is_dir() {
                    vfs.add_real_folder(parent, &path)
                } else {
                    vfs.add_file(parent, &path)
                };
                match result {
                    Ok(id) => {
                        added += 1;
                        self.selected = Some(id);
                    }
                    Err(err) => {
                        self.add_log(LogLevel::Error, i18n::format_err(Msg::AddFailed, err));
                    }
                }
            }
        }
        if added > 0 {
            self.add_log(LogLevel::Info, i18n::format_added(added));
            self.url_text = self.selected_url();
            self.state.save_vfs_async();
            if self.state.config.read().auto_copy_url_on_add {
                self.copy_selected_url(cx);
            }
        }
        cx.notify();
    }

    fn remove_selected(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.selected else {
            self.add_log(LogLevel::Warn, i18n::tr(Msg::NoSelection));
            return;
        };
        match self.state.vfs.write().remove(id) {
            Ok(()) => {
                self.selected = None;
                self.add_log(LogLevel::Info, i18n::tr(Msg::RemovedItem));
                self.url_text = self.state.config.read().public_base_url();
                self.state.save_vfs_async();
            }
            Err(err) => {
                self.add_log(LogLevel::Error, i18n::format_err(Msg::RemoveFailed, err));
            }
        }
        cx.notify();
    }

    fn open_new_folder_dialog(&mut self, cx: &mut Context<Self>) {
        let parent = self.target_parent();
        let input = cx.new(|cx| {
            let mut input = TextInput::new("New folder", cx);
            input.set_text("New folder", cx);
            input.start_blink(cx);
            input
        });
        self.dialog = DialogKind::NewFolder { parent, input };
        self.context_menu_open = false;
        cx.notify();
    }

    fn open_rename_dialog(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.selected else {
            self.add_log(LogLevel::Warn, i18n::tr(Msg::NoSelection));
            return;
        };
        if id == self.state.vfs.read().root_id() {
            self.add_log(LogLevel::Warn, i18n::tr(Msg::RootCannotRemove));
            return;
        }
        let current_name = self
            .state
            .vfs
            .read()
            .get(id)
            .map(|n| n.name.clone())
            .unwrap_or_default();

        let input = cx.new(|cx| {
            let mut input = TextInput::new(i18n::tr(Msg::RenamePrompt), cx);
            input.set_text(current_name, cx);
            input.start_blink(cx);
            input
        });
        self.dialog = DialogKind::Rename { id, input };
        self.context_menu_open = false;
        cx.notify();
    }

    fn open_new_link_dialog(&mut self, cx: &mut Context<Self>) {
        let parent = self.target_parent();
        let name_input = cx.new(|cx| {
            let mut input = TextInput::new(i18n::tr(Msg::RenamePrompt), cx);
            input.start_blink(cx);
            input
        });
        let url_input = cx.new(|cx| TextInput::new("https://", cx));
        self.dialog = DialogKind::NewLink {
            parent,
            name_input,
            url_input,
        };
        self.context_menu_open = false;
        cx.notify();
    }

    fn open_properties_dialog(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.selected else {
            self.add_log(LogLevel::Warn, i18n::tr(Msg::NoSelection));
            return;
        };
        self.dialog = DialogKind::Properties { id };
        self.context_menu_open = false;
        cx.notify();
    }

    fn open_settings(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        let cfg = self.state.config.read().clone();
        self.settings_port_input.update(cx, |i, cx| {
            i.set_text(cfg.port.to_string(), cx);
            i.start_blink(cx);
        });
        self.settings_max_conn_input.update(cx, |i, cx| {
            if let Some(n) = cfg.max_connections {
                i.set_text(n.to_string(), cx);
            } else {
                i.set_text("", cx);
            }
        });
        self.settings_upload_max_input.update(cx, |i, cx| {
            i.set_text(cfg.upload_max_mb.to_string(), cx);
        });
        self.settings_new_user_input.update(cx, |i, cx| i.clear(cx));
        self.settings_new_pass_input.update(cx, |i, cx| i.clear(cx));
        self.settings_tab = tab;
        self.settings_open_dropdown = None;
        self.settings_open = true;
        self.context_menu_open = false;
        cx.notify();
    }

    fn copy_all_logs(&mut self, _cx: &mut Context<Self>) {
        let text: String = self
            .logs
            .iter()
            .map(|l| format!("[{}] {}", l.time, l.text))
            .collect::<Vec<_>>()
            .join("\n");
        if let Ok(mut c) = arboard::Clipboard::new() {
            let _ = c.set_text(text);
            self.add_log(LogLevel::Info, "Copied log to clipboard");
        }
    }

    fn clear_logs(&mut self, cx: &mut Context<Self>) {
        self.logs.clear();
        cx.notify();
    }
}

// Rendering components
impl HfsApp {
    fn render_top_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = &self.theme;
        let mode_label = if self.expert_mode {
            i18n::tr(Msg::ExpertMode)
        } else {
            i18n::tr(Msg::EasyMode)
        };
        let (server_btn_label, server_btn_variant) = if self.server_running() {
            (
                i18n::tr(Msg::ServerOn),
                crate::ui::components::ButtonVariant::Danger,
            )
        } else {
            (
                i18n::tr(Msg::ServerOff),
                crate::ui::components::ButtonVariant::Success,
            )
        };

        let port_label = format!("{}: {}", i18n::tr(Msg::Port), self.state.config.read().port);

        div()
            .w_full()
            .px(px(8.0))
            .py(px(4.0))
            .bg(t.header_bg)
            .border_b_1()
            .border_color(t.card_border)
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    // Options Button (选项)
                    .child(
                        CustomButton::new("top-btn-options", i18n::tr(Msg::Options))
                            .outline()
                            .icon("⚙️")
                            .render(
                                t,
                                |this, _, cx| {
                                    this.open_settings(SettingsTab::General, cx);
                                },
                                cx,
                            ),
                    )
                    // Port Badge/Button
                    .child(
                        CustomButton::new("top-btn-port", port_label)
                            .outline()
                            .render(
                                t,
                                |this, _, cx| this.open_settings(SettingsTab::Server, cx),
                                cx,
                            ),
                    )
                    // Easy/Expert Mode Switcher
                    .child(
                        CustomButton::new("top-btn-mode", mode_label)
                            .outline()
                            .render(t, |this, _, cx| this.toggle_mode(cx), cx),
                    )
                    // Server ON/OFF Toggle Button
                    .child({
                        let mut btn = CustomButton::new("top-btn-server", server_btn_label);
                        match server_btn_variant {
                            crate::ui::components::ButtonVariant::Danger => btn = btn.danger(),
                            crate::ui::components::ButtonVariant::Success => btn = btn.success(),
                            _ => {}
                        }
                        btn.render(t, |this, _, cx| this.toggle_server(cx), cx)
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(t.text_secondary)
                            .child(self.status_text.clone()),
                    )
                    .child(render_badge(
                        if self.server_running() {
                            i18n::tr(Msg::Online)
                        } else {
                            i18n::tr(Msg::Offline)
                        },
                        t,
                        self.server_running(),
                    )),
            )
    }

    fn render_url_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let t = &self.theme;
        div()
            .w_full()
            .px(px(8.0))
            .py(px(4.0))
            .bg(t.panel_bg)
            .border_b_1()
            .border_color(t.card_border)
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(
                CustomButton::new("url-btn-open", i18n::tr(Msg::OpenInBrowser))
                    .outline()
                    .icon("🌐")
                    .render(t, |this, _, cx| this.open_in_browser(cx), cx),
            )
            .child(
                div()
                    .flex_1()
                    .px(px(8.0))
                    .py(px(4.0))
                    .rounded(px(5.0))
                    .bg(t.input_bg)
                    .border_1()
                    .border_color(t.input_border)
                    .text_size(px(12.5))
                    .text_color(t.text_primary)
                    .overflow_x_hidden()
                    .text_ellipsis()
                    .child(self.url_text.clone()),
            )
            .child(
                CustomButton::new("url-btn-copy", i18n::tr(Msg::CopyToClipboard))
                    .outline()
                    .icon("📋")
                    .render(t, |this, _, cx| this.copy_selected_url(cx), cx),
            )
    }

    fn render_status_bar(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let t = &self.theme;
        div()
            .w_full()
            .px(px(8.0))
            .py(px(4.0))
            .bg(t.header_bg)
            .border_t_1()
            .border_color(t.card_border)
            .flex()
            .items_center()
            .justify_between()
            .text_size(px(11.5))
            .text_color(t.text_muted)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .child(format!(
                        "{}: {}",
                        i18n::tr(Msg::Connections),
                        self.conn_count
                    ))
                    .child(format!(
                        "{}: {} ({})",
                        i18n::tr(Msg::OutBytes),
                        format_bytes(self.bytes_out),
                        format_speed(self.out_bps)
                    ))
                    .child(format!(
                        "{}: {} ({})",
                        i18n::tr(Msg::InBytes),
                        format_bytes(self.bytes_in),
                        format_speed(self.in_bps)
                    ))
                    .child(format!(
                        "{}: {}",
                        i18n::tr(Msg::Mode),
                        if self.expert_mode {
                            i18n::tr(Msg::ExpertMode)
                        } else {
                            i18n::tr(Msg::EasyMode)
                        }
                    ))
                    .child(format!(
                        "{}: {}",
                        i18n::tr(Msg::Language),
                        self.locale.native_name()
                    )),
            )
            .child(
                div()
                    .font_weight(FontWeight::MEDIUM)
                    .child(i18n::tr(Msg::StatusBarBrand)),
            )
    }
}

impl Render for HfsApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some((paths, as_folder)) = self.pending_paths.take() {
            self.add_paths(paths, as_folder, cx);
        }

        let theme = &self.theme;

        // VFS Props
        let vfs_props = VfsViewProps {
            theme,
            state: &self.state,
            selected: self.selected,
            expanded: &self.expanded,
            expert_mode: self.expert_mode,
            context_menu_open: self.context_menu_open,
            add_menu_open: self.vfs_add_menu_open,
        };

        // Log Props
        let log_props = LogViewProps {
            theme,
            search_input: &self.search_input,
            logs: &self.logs,
        };

        // Conn Props
        let conn_props = ConnViewProps {
            theme,
            connections: &self.connections,
        };

        // Graph Props
        let graph_props = GraphViewProps {
            theme,
            show_graph: self.show_graph,
            out_bps: self.out_bps,
            in_bps: self.in_bps,
            history: &self.graph_history,
        };

        div()
            .size_full()
            .relative()
            .bg(theme.bg)
            .text_color(theme.text_primary)
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                let list: Vec<PathBuf> = paths.paths().to_vec();
                this.add_paths(list, false, cx);
            }))
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .child(self.render_top_toolbar(cx))
                    .child(self.render_url_toolbar(cx))
                    .child(render_graph_pane(
                        graph_props,
                        |this, _, cx| this.toggle_graph(cx),
                        cx,
                    ))
                    // Resizable 3-pane Layout
                    .child(
                        div()
                            .flex_1()
                            .min_h(px(0.0))
                            .w_full()
                            .flex()
                            // Left Pane: VFS Tree
                            .child(
                                div()
                                    .w(px(self.left_panel_width))
                                    .min_w(px(160.0))
                                    .max_w(px(900.0))
                                    .h_full()
                                    .child(render_vfs_pane(
                                        vfs_props,
                                        |this, id, _, cx| {
                                            this.selected = Some(id);
                                            this.url_text = this.selected_url();
                                            this.vfs_add_menu_open = false;
                                            cx.notify();
                                        },
                                        |this, id, _, cx| {
                                            if !this.expanded.remove(&id.0) {
                                                this.expanded.insert(id.0);
                                            }
                                            cx.notify();
                                        },
                                        |this, _, cx| {
                                            this.vfs_add_menu_open = !this.vfs_add_menu_open;
                                            this.context_menu_open = false;
                                            cx.notify();
                                        },
                                        |this, _, cx| {
                                            this.vfs_add_menu_open = false;
                                            this.add_files(cx);
                                        },
                                        |this, _, cx| {
                                            this.vfs_add_menu_open = false;
                                            this.add_folder(cx);
                                        },
                                        |this, _, cx| {
                                            this.vfs_add_menu_open = false;
                                            this.open_new_folder_dialog(cx);
                                        },
                                        |this, _, cx| {
                                            this.vfs_add_menu_open = false;
                                            this.open_new_link_dialog(cx);
                                        },
                                        |this, _, cx| {
                                            this.vfs_add_menu_open = false;
                                            this.open_rename_dialog(cx);
                                        },
                                        |this, _, cx| {
                                            this.vfs_add_menu_open = false;
                                            this.remove_selected(cx);
                                        },
                                        |this, _, cx| {
                                            this.vfs_add_menu_open = false;
                                            this.open_properties_dialog(cx);
                                        },
                                        |this, id, _, cx| {
                                            if let Some(id) = id {
                                                this.selected = Some(id);
                                                this.url_text = this.selected_url();
                                            }
                                            this.vfs_add_menu_open = false;
                                            this.context_menu_open = true;
                                            cx.notify();
                                        },
                                        cx,
                                    )),
                            )
                            // Vertical Splitter Handle (between Left and Right)
                            .child(
                                div()
                                    .id("vfs-vertical-splitter")
                                    .w(px(5.0))
                                    .h_full()
                                    .cursor_col_resize()
                                    .bg(theme.card_border)
                                    .hover(|h| h.bg(theme.accent))
                                    .active(|a| a.bg(theme.accent))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, event: &gpui::MouseDownEvent, _window, cx| {
                                            let start_x: f32 = f32::from(event.position.x);
                                            this.resizing_splitter = Some(ResizingSplitter::LeftPanel {
                                                start_x,
                                                initial_width: this.left_panel_width,
                                            });
                                            this.vfs_add_menu_open = false;
                                            this.context_menu_open = false;
                                            cx.notify();
                                        }),
                                    ),
                            )
                            // Right Area: Log (top) + Splitter + Connections (bottom)
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.0))
                                    .h_full()
                                    .flex()
                                    .flex_col()
                                    // Right Top: Log Pane
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_h(px(60.0))
                                            .child(render_log_pane(
                                                log_props,
                                                |this, _, cx| this.clear_logs(cx),
                                                |this, _, cx| this.copy_all_logs(cx),
                                                cx,
                                            )),
                                    )
                                    // Horizontal Splitter Handle (between Log and Connections)
                                    .child(
                                        div()
                                            .id("log-conn-horizontal-splitter")
                                            .h(px(5.0))
                                            .w_full()
                                            .cursor_row_resize()
                                            .bg(theme.card_border)
                                            .hover(|h| h.bg(theme.accent))
                                            .active(|a| a.bg(theme.accent))
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, event: &gpui::MouseDownEvent, _window, cx| {
                                                    let start_y: f32 = f32::from(event.position.y);
                                                    this.resizing_splitter = Some(ResizingSplitter::BottomPanel {
                                                        start_y,
                                                        initial_height: this.bottom_panel_height,
                                                    });
                                                    this.vfs_add_menu_open = false;
                                                    this.context_menu_open = false;
                                                    cx.notify();
                                                }),
                                            ),
                                    )
                                    // Right Bottom: Connections Pane
                                    .child(
                                        div()
                                            .h(px(self.bottom_panel_height))
                                            .min_h(px(60.0))
                                            .max_h(px(700.0))
                                            .child(render_conn_pane(conn_props, cx)),
                                    ),
                            ),
                    )
                    .child(self.render_status_bar(cx)),
            )
            .child({
                if self.context_menu_open {
                    render_vfs_context_menu(
                        theme,
                        self.selected,
                        |this, _, cx| {
                            this.context_menu_open = false;
                            cx.notify();
                        },
                        |this, _, cx| {
                            this.context_menu_open = false;
                            this.add_files(cx);
                        },
                        |this, _, cx| {
                            this.context_menu_open = false;
                            this.add_folder(cx);
                        },
                        |this, _, cx| {
                            this.context_menu_open = false;
                            this.open_new_folder_dialog(cx);
                        },
                        |this, _, cx| {
                            this.context_menu_open = false;
                            this.open_new_link_dialog(cx);
                        },
                        |this, _, cx| {
                            this.context_menu_open = false;
                            this.open_rename_dialog(cx);
                        },
                        |this, _, cx| {
                            this.context_menu_open = false;
                            this.remove_selected(cx);
                        },
                        |this, _, cx| {
                            this.context_menu_open = false;
                            this.open_properties_dialog(cx);
                        },
                        |this, _, cx| {
                            this.context_menu_open = false;
                            this.open_in_browser(cx);
                        },
                        |this, _, cx| {
                            this.context_menu_open = false;
                            this.copy_selected_url(cx);
                        },
                        |this, _, cx| {
                            this.context_menu_open = false;
                            this.open_local_item(cx);
                        },
                        cx,
                    )
                } else {
                    div().into_any_element()
                }
            })
            .child({
                if self.vfs_add_menu_open {
                    div()
                        .absolute()
                        .inset_0()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.vfs_add_menu_open = false;
                                cx.notify();
                            }),
                        )
                        .on_mouse_down(
                            MouseButton::Right,
                            cx.listener(|this, _, _, cx| {
                                this.vfs_add_menu_open = false;
                                cx.notify();
                            }),
                        )
                        .into_any_element()
                } else {
                    div().into_any_element()
                }
            })
            .child({
                if let Some(resizing) = self.resizing_splitter {
                    let overlay = match resizing {
                        ResizingSplitter::LeftPanel { .. } => div().cursor_col_resize(),
                        ResizingSplitter::BottomPanel { .. } => div().cursor_row_resize(),
                    };
                    overlay
                        .absolute()
                        .inset_0()
                        .on_mouse_move(cx.listener(move |this, event: &gpui::MouseMoveEvent, _window, cx| {
                            match resizing {
                                ResizingSplitter::LeftPanel { start_x, initial_width } => {
                                    let cur_x: f32 = f32::from(event.position.x);
                                    let delta = cur_x - start_x;
                                    this.left_panel_width = (initial_width + delta).clamp(160.0, 900.0);
                                    cx.notify();
                                }
                                ResizingSplitter::BottomPanel { start_y, initial_height } => {
                                    let cur_y: f32 = f32::from(event.position.y);
                                    let delta = cur_y - start_y;
                                    this.bottom_panel_height = (initial_height - delta).clamp(60.0, 700.0);
                                    cx.notify();
                                }
                            }
                        }))
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(|this, _, _window, cx| {
                                this.resizing_splitter = None;
                                cx.notify();
                            }),
                        )
                        .into_any_element()
                } else {
                    div().into_any_element()
                }
            })
            .child(render_dialog_overlay(
                &self.dialog,
                theme,
                &self.state,
                self.selected,
                |this, _, cx| {
                    this.dialog = DialogKind::None;
                    cx.notify();
                },
                // on_confirm_new_folder
                |this, parent, name, _, cx| {
                    let result = this.state.vfs.write().add_virtual_folder(parent, &name);
                    match result {
                        Ok(id) => {
                            this.expanded.insert(parent.0);
                            this.selected = Some(id);
                            this.url_text = this.selected_url();
                            this.state.save_vfs_async();
                            this.add_log(LogLevel::Info, format!("{}: {name}", i18n::tr(Msg::CreatedFolder)));
                        }
                        Err(err) => {
                            this.add_log(LogLevel::Error, i18n::format_err(Msg::CreateFolderFailed, err));
                        }
                    }
                    this.dialog = DialogKind::None;
                    cx.notify();
                },
                // on_confirm_rename
                |this, id, name, _, cx| {
                    let result = this.state.vfs.write().rename(id, &name);
                    match result {
                        Ok(()) => {
                            this.url_text = this.selected_url();
                            this.state.save_vfs_async();
                            this.add_log(LogLevel::Info, format!("{}: {name}", i18n::tr(Msg::RenamedItem)));
                        }
                        Err(err) => {
                            this.add_log(LogLevel::Error, i18n::format_err(Msg::RenameFailed, err));
                        }
                    }
                    this.dialog = DialogKind::None;
                    cx.notify();
                },
                // on_confirm_new_link
                |this, parent, name, url, _, cx| {
                    let actual_name = if name.trim().is_empty() {
                        url.trim_end_matches('/').rsplit('/').next().unwrap_or("link")
                    } else {
                        &name
                    };
                    let result = this.state.vfs.write().add_link(parent, actual_name, &url);
                    match result {
                        Ok(id) => {
                            this.expanded.insert(parent.0);
                            this.selected = Some(id);
                            this.url_text = this.selected_url();
                            this.state.save_vfs_async();
                            this.add_log(LogLevel::Info, format!("Created link: {actual_name} -> {url}"));
                        }
                        Err(err) => {
                            this.add_log(LogLevel::Error, i18n::format_err(Msg::AddFailed, err));
                        }
                    }
                    this.dialog = DialogKind::None;
                    cx.notify();
                },
                // on_save_options
                |this, port, max_conn, upload_max, auto_copy, send_id, browse_local, allow_upload, protect_uploads, _, cx| {
                    {
                        let mut cfg = this.state.config.write();
                        cfg.port = port;
                        cfg.max_connections = max_conn;
                        cfg.upload_max_mb = upload_max;
                        cfg.auto_copy_url_on_add = auto_copy;
                        cfg.send_server_header = send_id;
                        cfg.open_in_browser_use_localhost = browse_local;
                        cfg.allow_upload = allow_upload;
                        cfg.protect_uploads = protect_uploads;
                    }
                    this.url_text = this.selected_url();
                    this.save_config();
                    this.dialog = DialogKind::None;
                    this.add_log(LogLevel::Info, "Saved server configuration");
                    cx.notify();
                },
                // on_add_account
                |this, user, pass, _, cx| {
                    {
                        let mut cfg = this.state.config.write();
                        cfg.accounts.push(crate::config::Account {
                            name: user.clone(),
                            password: pass,
                            enabled: true,
                        });
                    }
                    this.save_config();
                    this.add_log(LogLevel::Info, format!("Added account: {user}"));
                    cx.notify();
                },
                // on_remove_account
                |this, idx, _, cx| {
                    {
                        let mut cfg = this.state.config.write();
                        if idx < cfg.accounts.len() {
                            let removed = cfg.accounts.remove(idx);
                            this.add_log(LogLevel::Info, format!("Removed account: {}", removed.name));
                        }
                    }
                    this.save_config();
                    cx.notify();
                },
                // on_toggle_account
                |this, idx, _, cx| {
                    {
                        let mut cfg = this.state.config.write();
                        if let Some(acc) = cfg.accounts.get_mut(idx) {
                            acc.enabled = !acc.enabled;
                        }
                    }
                    this.save_config();
                    cx.notify();
                },
                cx,
            ))
            .child({
                if self.settings_open {
                    let props = SettingsViewProps {
                        theme,
                        theme_mode: self.theme_mode,
                        locale: self.locale,
                        expert_mode: self.expert_mode,
                        show_graph: self.show_graph,
                        state: &self.state,
                        active_tab: self.settings_tab,
                        open_dropdown: self.settings_open_dropdown,
                        port_input: &self.settings_port_input,
                        max_conn_input: &self.settings_max_conn_input,
                        upload_max_input: &self.settings_upload_max_input,
                        new_user_input: &self.settings_new_user_input,
                        new_pass_input: &self.settings_new_pass_input,
                    };
                    render_settings_overlay(
                        props,
                        |this, _, cx| {
                            this.settings_open = false;
                            this.settings_open_dropdown = None;
                            cx.notify();
                        },
                        |this, tab, _, cx| {
                            this.settings_tab = tab;
                            this.settings_open_dropdown = None;
                            cx.notify();
                        },
                        |this, dd, _, cx| {
                            this.settings_open_dropdown = dd;
                            cx.notify();
                        },
                        |this, loc, _, cx| {
                            this.locale = loc;
                            i18n::set(loc);
                            this.search_input.update(cx, |inp, cx| {
                                inp.set_placeholder(i18n::tr(Msg::Search), cx);
                            });
                            this.save_config();
                            this.settings_open_dropdown = None;
                            cx.notify();
                        },
                        |this, mode, _, cx| {
                            this.theme_mode = mode;
                            this.theme = Theme::for_mode(mode);
                            this.save_config();
                            this.settings_open_dropdown = None;
                            cx.notify();
                        },
                        |this, exp, _, cx| {
                            this.expert_mode = exp;
                            this.save_config();
                            this.settings_open_dropdown = None;
                            cx.notify();
                        },
                        |this, show_g, _, cx| {
                            this.show_graph = show_g;
                            this.save_config();
                            this.settings_open_dropdown = None;
                            cx.notify();
                        },
                        |this, _, cx| {
                            {
                                let mut cfg = this.state.config.write();
                                cfg.auto_copy_url_on_add = !cfg.auto_copy_url_on_add;
                            }
                            this.save_config();
                            cx.notify();
                        },
                        |this, _, cx| {
                            {
                                let mut cfg = this.state.config.write();
                                cfg.send_server_header = !cfg.send_server_header;
                            }
                            this.save_config();
                            cx.notify();
                        },
                        |this, _, cx| {
                            {
                                let mut cfg = this.state.config.write();
                                cfg.open_in_browser_use_localhost = !cfg.open_in_browser_use_localhost;
                            }
                            this.url_text = this.selected_url();
                            this.save_config();
                            cx.notify();
                        },
                        |this, (allow_up, prot_up), _, cx| {
                            {
                                let mut cfg = this.state.config.write();
                                cfg.allow_upload = allow_up;
                                cfg.protect_uploads = prot_up;
                            }
                            this.save_config();
                            this.settings_open_dropdown = None;
                            cx.notify();
                        },
                        |this, _, cx| {
                            let port = this.settings_port_input.read(cx).text().trim().parse::<u16>().unwrap_or(8080);
                            let max_conn = this.settings_max_conn_input.read(cx).text().trim().parse::<usize>().ok();
                            let upload_max = this.settings_upload_max_input.read(cx).text().trim().parse::<u64>().unwrap_or(512);
                            {
                                let mut cfg = this.state.config.write();
                                cfg.port = port;
                                cfg.max_connections = max_conn;
                                cfg.upload_max_mb = upload_max;
                            }
                            this.url_text = this.selected_url();
                            this.save_config();
                            this.add_log(LogLevel::Info, "Applied server network settings");
                            cx.notify();
                        },
                        |this, _, cx| {
                            let user = this.settings_new_user_input.read(cx).text().trim().to_string();
                            let pass = this.settings_new_pass_input.read(cx).text().trim().to_string();
                            if !user.is_empty() {
                                {
                                    let mut cfg = this.state.config.write();
                                    cfg.accounts.push(crate::config::Account {
                                        name: user.clone(),
                                        password: pass,
                                        enabled: true,
                                    });
                                }
                                this.save_config();
                                this.settings_new_user_input.update(cx, |i, cx| i.clear(cx));
                                this.settings_new_pass_input.update(cx, |i, cx| i.clear(cx));
                                this.add_log(LogLevel::Info, format!("Added account: {user}"));
                                cx.notify();
                            }
                        },
                        |this, idx, _, cx| {
                            {
                                let mut cfg = this.state.config.write();
                                if idx < cfg.accounts.len() {
                                    let removed = cfg.accounts.remove(idx);
                                    this.add_log(LogLevel::Info, format!("Removed account: {}", removed.name));
                                }
                            }
                            this.save_config();
                            cx.notify();
                        },
                        |this, idx, _, cx| {
                            {
                                let mut cfg = this.state.config.write();
                                if let Some(acc) = cfg.accounts.get_mut(idx) {
                                    acc.enabled = !acc.enabled;
                                }
                            }
                            this.save_config();
                            cx.notify();
                        },
                        cx,
                    )
                } else {
                    div().into_any_element()
                }
            })
    }
}
