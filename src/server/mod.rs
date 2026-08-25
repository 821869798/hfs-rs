//! Shared app state, server lifecycle, connection tracking, UI event bus.

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

use parking_lot::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::vfs::Vfs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Http,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Log {
        level: LogLevel,
        text: String,
    },
    ServerStatus(ServerStatus),
    ServerStarted {
        addr: String,
    },
    ServerStopped,
    ConnectionUpsert(ConnInfo),
    ConnectionRemoved {
        id: String,
    },
    Stats {
        bytes_out: u64,
        bytes_in: u64,
        connections: usize,
    },
    BandwidthSample {
        out_bps: f64,
        in_bps: f64,
    },
    PathsPicked {
        paths: Vec<std::path::PathBuf>,
        as_folder: bool,
    },
}

#[derive(Debug, Clone)]
pub struct ConnInfo {
    pub id: String,
    pub peer: String,
    pub file: String,
    pub status: String,
    pub speed: f64,
    pub progress: f32,
    pub bytes_sent: u64,
    pub bytes_total: u64,
    /// For UI throttle only (ms since unix epoch).
    pub last_ui_emit_ms: u64,
}

#[derive(Default)]
pub struct EventBus {
    queue: RwLock<Vec<AppEvent>>,
}

impl EventBus {
    pub fn push(&self, ev: AppEvent) {
        self.queue.write().push(ev);
    }

    pub fn drain(&self) -> Vec<AppEvent> {
        std::mem::take(&mut *self.queue.write())
    }

    pub fn log(&self, level: LogLevel, text: impl Into<String>) {
        self.push(AppEvent::Log {
            level,
            text: text.into(),
        });
    }
}

pub struct ServerMetrics {
    pub bytes_out: AtomicU64,
    pub bytes_in: AtomicU64,
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self {
            bytes_out: AtomicU64::new(0),
            bytes_in: AtomicU64::new(0),
        }
    }
}

pub struct ServerHandle {
    status: RwLock<ServerStatus>,
    cancel: RwLock<Option<CancellationToken>>,
    pub metrics: Arc<ServerMetrics>,
    pub connections: Arc<RwLock<HashMap<String, ConnInfo>>>,
    running: AtomicBool,
}

impl Default for ServerHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerHandle {
    pub fn new() -> Self {
        Self {
            status: RwLock::new(ServerStatus::Stopped),
            cancel: RwLock::new(None),
            metrics: Arc::new(ServerMetrics::default()),
            connections: Arc::new(RwLock::new(HashMap::new())),
            running: AtomicBool::new(false),
        }
    }

    pub fn status(&self) -> ServerStatus {
        *self.status.read()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn set_status(&self, bus: &EventBus, status: ServerStatus) {
        *self.status.write() = status;
        bus.push(AppEvent::ServerStatus(status));
        match status {
            ServerStatus::Running => self.running.store(true, Ordering::SeqCst),
            ServerStatus::Stopped => self.running.store(false, Ordering::SeqCst),
            _ => {}
        }
    }

    pub fn stop(&self, bus: &EventBus) {
        if let Some(token) = self.cancel.write().take() {
            self.set_status(bus, ServerStatus::Stopping);
            bus.log(LogLevel::Info, "Stopping server…");
            token.cancel();
        }
    }
}

/// Process-wide shared state between UI and HTTP server.
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub vfs: Arc<RwLock<Vfs>>,
    pub server: Arc<ServerHandle>,
    pub bus: Arc<EventBus>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        let config = AppConfig::load_or_default();
        let state = Arc::new(Self {
            config: Arc::new(RwLock::new(config)),
            vfs: Arc::new(RwLock::new(Vfs::new())),
            server: Arc::new(ServerHandle::new()),
            bus: Arc::new(EventBus::default()),
        });
        state.load_vfs();
        state
    }

    pub fn start_server(self: &Arc<Self>) {
        if self.server.is_running()
            || matches!(
                self.server.status(),
                ServerStatus::Starting | ServerStatus::Running
            )
        {
            self.bus.log(LogLevel::Warn, "Server already running");
            return;
        }

        let addr = self.config.read().listen_addr();
        self.server.set_status(&self.bus, ServerStatus::Starting);
        self.bus
            .log(LogLevel::Info, format!("Starting server on {addr}…"));

        let token = CancellationToken::new();
        *self.server.cancel.write() = Some(token.clone());

        let state = Arc::clone(self);
        thread::Builder::new()
            .name("hfs-http".into())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .thread_name("hfs-tokio")
                    .build()
                {
                    Ok(rt) => rt,
                    Err(err) => {
                        state
                            .bus
                            .log(LogLevel::Error, format!("Tokio runtime failed: {err}"));
                        state.server.set_status(&state.bus, ServerStatus::Stopped);
                        return;
                    }
                };

                rt.block_on(async move {
                    if let Err(err) = crate::http::serve(state.clone(), token).await {
                        state
                            .bus
                            .log(LogLevel::Error, format!("Server error: {err}"));
                    }
                    state.server.connections.write().clear();
                    state.server.set_status(&state.bus, ServerStatus::Stopped);
                    state.bus.push(AppEvent::ServerStopped);
                    state.bus.log(LogLevel::Info, "Server stopped");
                });
            })
            .expect("spawn http thread");
    }

    pub fn stop_server(&self) {
        self.server.stop(&self.bus);
    }

    pub fn upsert_conn(&self, mut info: ConnInfo) {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let mut conns = self.server.connections.write();
        let should_emit = match conns.get(&info.id) {
            Some(prev) => {
                // Emit at most ~5 Hz, or on finish-like progress jumps.
                now_ms.saturating_sub(prev.last_ui_emit_ms) >= 200
                    || (info.progress - prev.progress).abs() >= 0.05
                    || info.bytes_sent >= info.bytes_total
                    || info.status != prev.status
            }
            None => true,
        };
        info.last_ui_emit_ms = if should_emit {
            now_ms
        } else {
            conns
                .get(&info.id)
                .map(|c| c.last_ui_emit_ms)
                .unwrap_or(now_ms)
        };
        conns.insert(info.id.clone(), info.clone());
        drop(conns);
        if should_emit {
            self.bus.push(AppEvent::ConnectionUpsert(info));
        }
    }

    pub fn remove_conn(&self, id: &str) {
        self.server.connections.write().remove(id);
        self.bus
            .push(AppEvent::ConnectionRemoved { id: id.to_string() });
    }

    pub fn new_conn_id() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn vfs_path() -> std::path::PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("hfs-rs.vfs.json")))
            .unwrap_or_else(|| std::path::PathBuf::from("hfs-rs.vfs.json"))
    }

    pub fn load_vfs(&self) {
        let path = Self::vfs_path();
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(vfs) = serde_json::from_str::<crate::vfs::Vfs>(&raw) {
                *self.vfs.write() = vfs;
                self.bus.log(
                    LogLevel::Info,
                    format!("Loaded VFS from {}", path.display()),
                );
            }
        }
    }

    pub fn save_vfs(&self) {
        let path = Self::vfs_path();
        let snapshot = self.vfs.read().clone();
        // Disk IO without holding the lock long: clone under lock, write outside.
        match serde_json::to_string_pretty(&snapshot) {
            Ok(raw) => {
                if let Err(err) = std::fs::write(&path, raw) {
                    self.bus
                        .log(LogLevel::Error, format!("Save VFS failed: {err}"));
                }
            }
            Err(err) => self
                .bus
                .log(LogLevel::Error, format!("Serialize VFS failed: {err}")),
        }
    }

    pub fn save_vfs_async(self: &Arc<Self>) {
        let state = Arc::clone(self);
        std::thread::Builder::new()
            .name("hfs-vfs-save".into())
            .spawn(move || state.save_vfs())
            .ok();
    }

    pub fn publish_stats(&self) {
        let bytes_out = self.server.metrics.bytes_out.load(Ordering::Relaxed);
        let bytes_in = self.server.metrics.bytes_in.load(Ordering::Relaxed);
        let connections = self.server.connections.read().len();
        self.bus.push(AppEvent::Stats {
            bytes_out,
            bytes_in,
            connections,
        });
    }
}

/// Background helper: periodically emit stats while running.
pub fn spawn_stats_poller(state: Arc<AppState>, token: CancellationToken) {
    tokio::spawn(async move {
        let mut last_out = state.server.metrics.bytes_out.load(Ordering::Relaxed);
        let mut last_in = state.server.metrics.bytes_in.load(Ordering::Relaxed);
        let mut last_instant = std::time::Instant::now();
        loop {
            tokio::select! {
                _ = token.cancelled() => break,
                _ = tokio::time::sleep(Duration::from_millis(500)) => {
                    state.publish_stats();
                    let now_out = state.server.metrics.bytes_out.load(Ordering::Relaxed);
                    let now_in = state.server.metrics.bytes_in.load(Ordering::Relaxed);
                    let elapsed = last_instant.elapsed().as_secs_f64().max(0.001);
                    let out_bps = (now_out.saturating_sub(last_out)) as f64 / elapsed;
                    let in_bps = (now_in.saturating_sub(last_in)) as f64 / elapsed;
                    // Update per-connection approximate speed from progress deltas.
                    {
                        let mut conns = state.server.connections.write();
                        let n = conns.len().max(1) as f64;
                        for conn in conns.values_mut() {
                            if out_bps > 1.0 {
                                conn.speed = out_bps / n;
                            }
                        }
                    }
                    state.bus.push(AppEvent::BandwidthSample { out_bps, in_bps });
                    last_out = now_out;
                    last_in = now_in;
                    last_instant = std::time::Instant::now();
                }
            }
        }
    });
}

pub fn peer_to_string(addr: Option<SocketAddr>) -> String {
    addr.map(|a| a.ip().to_string())
        .unwrap_or_else(|| "-".into())
}
