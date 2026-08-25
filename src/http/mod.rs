//! HTTP server and request handlers (hyper 1.x).

use std::{
    convert::Infallible,
    fs,
    io::SeekFrom,
    net::SocketAddr,
    path::Path,
    pin::Pin,
    sync::{Arc, atomic::Ordering},
    task::{Context, Poll},
};

use anyhow::Context as _;
use bytes::Bytes;
use futures_util::StreamExt;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::{
    Method, Request, Response, StatusCode,
    body::{Frame, Incoming},
    header,
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as AutoBuilder;
use tokio::fs::File as TokioFile;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::{io::ReaderStream, sync::CancellationToken};

use crate::server::{
    AppState, ConnInfo, LogLevel, ServerStatus, peer_to_string, spawn_stats_poller,
};
use crate::util::{format_bytes, percent_encode_path_segment};
use crate::vfs::{DirEntry, NodeKind, ResolveResult};

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type RespBody = http_body_util::combinators::BoxBody<Bytes, BoxError>;

pub async fn serve(state: Arc<AppState>, token: CancellationToken) -> anyhow::Result<()> {
    let listen = state.config.read().listen_addr();
    let listener = tokio::net::TcpListener::bind(&listen)
        .await
        .with_context(|| format!("bind {listen}"))?;
    let local = listener
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| listen.clone());

    state.server.set_status(&state.bus, ServerStatus::Running);
    state.bus.push(crate::server::AppEvent::ServerStarted {
        addr: local.clone(),
    });
    state
        .bus
        .log(LogLevel::Info, format!("Server is ON ? listening {local}"));

    spawn_stats_poller(state.clone(), token.clone());

    loop {
        tokio::select! {
            _ = token.cancelled() => break,
            accepted = listener.accept() => {
                let (stream, peer) = match accepted {
                    Ok(v) => v,
                    Err(err) => {
                        state.bus.log(LogLevel::Error, format!("accept error: {err}"));
                        continue;
                    }
                };

                if let Some(max) = state.config.read().max_connections {
                    if state.server.connections.read().len() >= max {
                        state.bus.log(
                            LogLevel::Warn,
                            format!("max connections reached, drop {peer}"),
                        );
                        continue;
                    }
                }

                let state = state.clone();
                let token = token.clone();
                tokio::spawn(async move {
                    let io = TokioIo::new(stream);
                    let service = hyper::service::service_fn(move |req| {
                        let state = state.clone();
                        async move {
                            Ok::<_, Infallible>(handle_request(state, req, Some(peer)).await)
                        }
                    });

                    let builder = AutoBuilder::new(TokioExecutor::new());
                    let conn = builder.serve_connection(io, service);
                    tokio::pin!(conn);

                    tokio::select! {
                        _ = token.cancelled() => {}
                        _ = &mut conn => {}
                    }
                });
            }
        }
    }

    Ok(())
}

async fn handle_upload(
    state: Arc<AppState>,
    req: Request<Incoming>,
    _peer: Option<SocketAddr>,
) -> Response<RespBody> {
    let upload_dir = {
        let vfs = state.vfs.read();
        let resolved = vfs.resolve(req.uri().path());
        match resolved {
            ResolveResult::Node(id) => {
                let n = vfs.get(id);
                if n.map(|n| n.kind == NodeKind::RealFolder).unwrap_or(false) {
                    n.and_then(|n| n.resource.clone())
                } else {
                    None
                }
            }
            ResolveResult::DiskDir { path, .. } => Some(path),
            _ => None,
        }
    };

    let upload_dir = match upload_dir {
        Some(d) => d,
        None => return html_status(StatusCode::BAD_REQUEST, "400", "No writable directory."),
    };

    let boundary = (|| {
        let ct = req.headers().get("content-type")?.to_str().ok()?;
        if !ct.starts_with("multipart/form-data") {
            return None;
        }
        ct.split(';')
            .nth(1)?
            .split('=')
            .nth(1)
            .map(|s| s.trim().to_string())
    })();
    let boundary = match boundary {
        Some(b) => b,
        None => {
            return html_status(
                StatusCode::BAD_REQUEST,
                "400",
                "Missing multipart boundary.",
            );
        }
    };

    let body = req.into_body();
    let bytes = match body.collect().await {
        Ok(b) => b.to_bytes(),
        Err(_) => {
            return html_status(
                StatusCode::INTERNAL_SERVER_ERROR,
                "500",
                "Failed to read body.",
            );
        }
    };

    let body_str = String::from_utf8_lossy(&bytes).to_string();
    let mut files = Vec::new();
    for part in body_str.split(&format!("--{boundary}")) {
        if !part.contains("filename=") {
            continue;
        }
        let filename = part
            .split("filename=\"")
            .nth(1)
            .and_then(|s| s.split('"').next());
        let content = part.split("\r\n\r\n").nth(1).unwrap_or("").trim();
        if let Some(name) = filename {
            if name.is_empty() || content.is_empty() {
                continue;
            }
            files.push((name.to_string(), content.as_bytes().to_vec()));
        }
    }

    if files.is_empty() {
        return html_status(StatusCode::BAD_REQUEST, "400", "No files uploaded.");
    }

    for (name, data) in files {
        let target = upload_dir.join(&name);
        if let Err(err) = std::fs::write(&target, data) {
            return html_status(
                StatusCode::INTERNAL_SERVER_ERROR,
                "500",
                &format!("Write failed: {err}"),
            );
        }
        state
            .bus
            .log(LogLevel::Info, format!("Uploaded {name} to {target:?}"));
    }

    html_status(StatusCode::OK, "200", "Upload complete.")
}
async fn handle_request(
    state: Arc<AppState>,
    req: Request<Incoming>,
    peer: Option<SocketAddr>,
) -> Response<RespBody> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let peer_s = peer_to_string(peer);

    state
        .bus
        .log(LogLevel::Http, format!("{peer_s} {method} {path}"));

    if path == "/favicon.ico" {
        return text_response(StatusCode::NOT_FOUND, "no favicon");
    }
    if path == "/~style.css" {
        return bytes_response(
            StatusCode::OK,
            "text/css; charset=utf-8",
            STYLE_CSS.as_bytes(),
        );
    }

    let resolved = {
        let vfs = state.vfs.read();
        vfs.resolve(&path)
    };

    match method {
        Method::GET | Method::HEAD => match &resolved {
            ResolveResult::NotFound => html_status(
                StatusCode::NOT_FOUND,
                "404 Not Found",
                "Resource not found.",
            ),
            ResolveResult::Forbidden => {
                html_status(StatusCode::FORBIDDEN, "403 Forbidden", "Forbidden.")
            }
            other if other.is_dir(&state.vfs.read()) => {
                let vfs = state.vfs.read();
                let entries = vfs.list_dir(other);
                let html =
                    render_directory(&path, &path, &entries, state.config.read().allow_upload);
                let mut resp =
                    bytes_response(StatusCode::OK, "text/html; charset=utf-8", html.as_bytes());
                if method == Method::HEAD {
                    *resp.body_mut() = empty_body();
                }
                maybe_server_header(&state, &mut resp);
                resp
            }
            other => {
                let file_path = {
                    let vfs = state.vfs.read();
                    vfs.file_path_for(other)
                };
                match file_path {
                    Some(p) => serve_file(&state, &req, &p, &peer_s, method == Method::HEAD).await,
                    None => html_status(StatusCode::NOT_FOUND, "404 Not Found", "Not a file."),
                }
            }
        },
        Method::POST => {
            if !state.config.read().allow_upload {
                return html_status(
                    StatusCode::FORBIDDEN,
                    "403 Forbidden",
                    "Uploads not allowed.",
                );
            }
            // Check upload auth if accounts exist and protect_uploads is enabled.
            if state.config.read().protect_uploads && !state.config.read().accounts.is_empty() {
                if !check_basic_auth(&state, &req) {
                    return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .header(header::WWW_AUTHENTICATE, "Basic realm=\"HFS-RS Upload\"")
                        .body(empty_body())
                        .unwrap_or_else(|_| {
                            text_response(StatusCode::INTERNAL_SERVER_ERROR, "error")
                        });
                }
            }
            handle_upload(state, req, peer).await
        }
        _ => {
            let mut resp = text_response(StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed");
            resp.headers_mut()
                .insert(header::ALLOW, header::HeaderValue::from_static("GET, HEAD"));
            resp
        }
    }
}

async fn serve_file(
    state: &Arc<AppState>,
    req: &Request<Incoming>,
    path: &Path,
    peer: &str,
    head_only: bool,
) -> Response<RespBody> {
    let meta = match fs::metadata(path) {
        Ok(m) if m.is_file() => m,
        _ => return html_status(StatusCode::NOT_FOUND, "404 Not Found", "File missing."),
    };
    let file_len = meta.len();
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    let range_header = req
        .headers()
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let (start, end, status) = match parse_range(range_header.as_deref(), file_len) {
        RangeParse::Full => (0u64, file_len.saturating_sub(1), StatusCode::OK),
        RangeParse::Partial { start, end } => (start, end, StatusCode::PARTIAL_CONTENT),
        RangeParse::Unsatisfiable => {
            let mut resp =
                text_response(StatusCode::RANGE_NOT_SATISFIABLE, "Range Not Satisfiable");
            let val = format!("bytes */{file_len}");
            if let Ok(hv) = header::HeaderValue::from_str(&val) {
                resp.headers_mut().insert(header::CONTENT_RANGE, hv);
            }
            return resp;
        }
    };

    let content_len = end.saturating_sub(start).saturating_add(1);
    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "download".into());

    let conn_id = AppState::new_conn_id();
    state.upsert_conn(ConnInfo {
        id: conn_id.clone(),
        peer: peer.to_string(),
        file: file_name.clone(),
        status: if status == StatusCode::PARTIAL_CONTENT {
            "Partial".into()
        } else {
            "Downloading".into()
        },
        speed: 0.0,
        progress: if file_len == 0 {
            1.0
        } else {
            start as f32 / file_len as f32
        },
        bytes_sent: 0,
        bytes_total: content_len,
        last_ui_emit_ms: 0,
    });

    if head_only {
        state.remove_conn(&conn_id);
        let mut builder = Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, &mime)
            .header(header::CONTENT_LENGTH, content_len)
            .header(header::ACCEPT_RANGES, "bytes")
            .header(
                header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{file_name}\""),
            );
        if status == StatusCode::PARTIAL_CONTENT {
            builder = builder.header(
                header::CONTENT_RANGE,
                format!("bytes {start}-{end}/{file_len}"),
            );
        }
        let mut resp = builder
            .body(empty_body())
            .unwrap_or_else(|_| text_response(StatusCode::INTERNAL_SERVER_ERROR, "error"));
        maybe_server_header(state, &mut resp);
        return resp;
    }

    let mut file = match TokioFile::open(path).await {
        Ok(f) => f,
        Err(err) => {
            state.remove_conn(&conn_id);
            return html_status(
                StatusCode::INTERNAL_SERVER_ERROR,
                "500",
                &format!("Open failed: {err}"),
            );
        }
    };

    if start > 0 {
        if let Err(err) = file.seek(SeekFrom::Start(start)).await {
            state.remove_conn(&conn_id);
            return html_status(
                StatusCode::INTERNAL_SERVER_ERROR,
                "500",
                &format!("Seek failed: {err}"),
            );
        }
    }

    let limited = file.take(content_len);
    let state_c = Arc::clone(state);
    let conn_id_c = conn_id.clone();
    let stream = ReaderStream::new(limited);
    let mut sent = 0u64;
    let mapped = stream.map(move |chunk| match chunk {
        Ok(bytes) => {
            let n = bytes.len() as u64;
            sent += n;
            state_c
                .server
                .metrics
                .bytes_out
                .fetch_add(n, Ordering::Relaxed);
            {
                // Update connection progress with throttled UI events.
                let mut info = state_c.server.connections.read().get(&conn_id_c).cloned();
                if let Some(ref mut conn) = info {
                    conn.bytes_sent = sent;
                    conn.progress = if content_len == 0 {
                        1.0
                    } else {
                        (sent as f32 / content_len as f32).min(1.0)
                    };
                    state_c.upsert_conn(conn.clone());
                }
            }
            Ok(Frame::data(bytes))
        }
        Err(err) => Err(Box::new(err) as BoxError),
    });

    let body_stream = DisconnectGuardStream {
        inner: mapped,
        state: Arc::clone(state),
        conn_id: conn_id.clone(),
        file_name: file_name.clone(),
        peer: peer.to_string(),
        finished: false,
    };

    let body = BodyExt::boxed(StreamBody::new(body_stream));
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, mime)
        .header(header::CONTENT_LENGTH, content_len)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{file_name}\""),
        );

    if status == StatusCode::PARTIAL_CONTENT {
        builder = builder.header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{file_len}"),
        );
    }

    let mut resp = builder
        .body(body)
        .unwrap_or_else(|_| text_response(StatusCode::INTERNAL_SERVER_ERROR, "error"));
    maybe_server_header(state, &mut resp);

    state.bus.log(
        LogLevel::Http,
        format!("{peer} -> {file_name} ({})", format_bytes(content_len)),
    );

    resp
}

struct DisconnectGuardStream<S> {
    inner: S,
    state: Arc<AppState>,
    conn_id: String,
    file_name: String,
    peer: String,
    finished: bool,
}

impl<S> futures_core::Stream for DisconnectGuardStream<S>
where
    S: futures_core::Stream<Item = Result<Frame<Bytes>, BoxError>> + Unpin,
{
    type Item = Result<Frame<Bytes>, BoxError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let polled = Pin::new(&mut self.inner).poll_next(cx);
        if let Poll::Ready(None) = &polled {
            self.finished = true;
            self.state.remove_conn(&self.conn_id);
            self.state.bus.log(
                LogLevel::Http,
                format!("{} done {}", self.peer, self.file_name),
            );
        }
        polled
    }
}

impl<S> Drop for DisconnectGuardStream<S> {
    fn drop(&mut self) {
        if !self.finished {
            self.state.remove_conn(&self.conn_id);
            self.state.bus.log(
                LogLevel::Http,
                format!("{} aborted {}", self.peer, self.file_name),
            );
        }
    }
}

enum RangeParse {
    Full,
    Partial { start: u64, end: u64 },
    Unsatisfiable,
}

fn parse_range(header: Option<&str>, file_len: u64) -> RangeParse {
    let Some(h) = header else {
        return RangeParse::Full;
    };
    let h = h.trim();
    let Some(rest) = h.strip_prefix("bytes=") else {
        return RangeParse::Full;
    };
    let spec = rest.split(',').next().unwrap_or("").trim();
    if spec.is_empty() {
        return RangeParse::Full;
    }
    let (start_s, end_s) = match spec.split_once('-') {
        Some(v) => v,
        None => return RangeParse::Full,
    };

    if file_len == 0 {
        return RangeParse::Unsatisfiable;
    }

    if start_s.is_empty() {
        let n: u64 = end_s.parse().unwrap_or(0);
        if n == 0 {
            return RangeParse::Unsatisfiable;
        }
        let start = file_len.saturating_sub(n);
        return RangeParse::Partial {
            start,
            end: file_len - 1,
        };
    }

    let start: u64 = match start_s.parse() {
        Ok(v) => v,
        Err(_) => return RangeParse::Unsatisfiable,
    };
    if start >= file_len {
        return RangeParse::Unsatisfiable;
    }
    let end = if end_s.is_empty() {
        file_len - 1
    } else {
        match end_s.parse::<u64>() {
            Ok(v) => v.min(file_len - 1),
            Err(_) => return RangeParse::Unsatisfiable,
        }
    };
    if end < start {
        return RangeParse::Unsatisfiable;
    }
    RangeParse::Partial { start, end }
}

fn maybe_server_header(state: &AppState, resp: &mut Response<RespBody>) {
    if state.config.read().send_server_header {
        resp.headers_mut().insert(
            header::SERVER,
            header::HeaderValue::from_static("HFS-RS/0.1"),
        );
    }
}

fn empty_body() -> RespBody {
    Full::new(Bytes::new())
        .map_err(|never| match never {})
        .boxed()
}

fn text_response(status: StatusCode, body: &str) -> Response<RespBody> {
    bytes_response(status, "text/plain; charset=utf-8", body.as_bytes())
}

fn bytes_response(status: StatusCode, content_type: &str, body: &[u8]) -> Response<RespBody> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, body.len())
        .body(
            Full::new(Bytes::copy_from_slice(body))
                .map_err(|never| match never {})
                .boxed(),
        )
        .unwrap_or_else(|_| {
            Response::new(
                Full::new(Bytes::from_static(b"err"))
                    .map_err(|never| match never {})
                    .boxed(),
            )
        })
}

fn html_status(status: StatusCode, title: &str, msg: &str) -> Response<RespBody> {
    let html = format!(
        concat!(
            "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>{title}</title>",
            "<link rel=\"stylesheet\" href=\"/~style.css\"></head>",
            "<body><div id=\"wrapper\"><h1>{title}</h1><p>{msg}</p>",
            "<p><a href=\"/\">Home</a></p></div></body></html>"
        ),
        title = title,
        msg = msg
    );
    bytes_response(status, "text/html; charset=utf-8", html.as_bytes())
}

fn render_directory(
    title: &str,
    url_path: &str,
    entries: &[DirEntry],
    allow_upload: bool,
) -> String {
    let mut rows = String::new();
    if url_path != "/" {
        let parent = parent_path(url_path);
        rows.push_str(&format!(
            "<div class=\"item dir\"><a href=\"{parent}\"><span class=\"icon\">^</span><span class=\"name\">..</span></a><span class=\"mtime\"></span><span class=\"size\">?</span></div>"
        ));
    }
    for e in entries {
        let href = join_child_url(url_path, &e.name, e.is_dir);
        let size = e
            .size
            .map(format_bytes)
            .unwrap_or_else(|| if e.is_dir { "?".into() } else { "0 B".into() });
        let mtime = e.mtime.clone().unwrap_or_default();
        let kind = if e.is_dir { "dir" } else { "file" };
        let icon = if e.is_dir { "[D]" } else { "[F]" };
        rows.push_str(&format!(
            "<div class=\"item {kind}\"><a href=\"{href}\"><span class=\"icon\">{icon}</span><span class=\"name\">{name}</span></a><span class=\"mtime\">{mtime}</span><span class=\"size\">{size}</span></div>",
            kind = kind,
            href = href,
            icon = icon,
            name = html_escape(&e.name),
            mtime = html_escape(&mtime),
            size = size
        ));
    }

    let upload_form = if allow_upload {
        "<div id=\"upload-form\"><form method=\"post\" enctype=\"multipart/form-data\"><input type=\"file\" name=\"file\" multiple/><button type=\"submit\">Upload</button></form></div>".to_string()
    } else {
        String::new()
    };

    format!(
        concat!(
            "<!DOCTYPE html><html><head><meta charset=\"utf-8\">",
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">",
            "<title>HFS-RS {title}</title>",
            "<link rel=\"stylesheet\" href=\"/~style.css\"></head><body><div id=\"wrapper\">",
            "<div id=\"menu-panel\"><div class=\"brand\">HFS-RS</div>",
            "<div class=\"path\">{path}</div></div>",
            "{upload_form}",
            "<div id=\"files\">{rows}</div>",
            "<div id=\"serverinfo\">Inspired by HFS2 ? Rust + GPUI</div>",
            "</div></body></html>"
        ),
        title = html_escape(title),
        path = html_escape(url_path),
        upload_form = upload_form,
        rows = rows
    )
}

fn parent_path(url_path: &str) -> String {
    let trimmed = url_path.trim_end_matches('/');
    if trimmed.is_empty() || trimmed == "/" {
        return "/".into();
    }
    match trimmed.rsplit_once('/') {
        Some(("", _)) => "/".into(),
        Some((prefix, _)) => format!("{prefix}/"),
        None => "/".into(),
    }
}

fn join_child_url(base: &str, name: &str, is_dir: bool) -> String {
    let enc = percent_encode_path_segment(name);
    let base = if base.ends_with('/') {
        base.to_string()
    } else {
        format!("{base}/")
    };
    if is_dir {
        format!("{base}{enc}/")
    } else {
        format!("{base}{enc}")
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn check_basic_auth(state: &AppState, req: &Request<Incoming>) -> bool {
    let Some(auth) = req.headers().get(header::AUTHORIZATION) else {
        return false;
    };
    let Some(auth_str) = auth.to_str().ok() else {
        return false;
    };
    let Some(creds) = auth_str.strip_prefix("Basic ") else {
        return false;
    };
    let Ok(decoded) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, creds)
    else {
        return false;
    };
    let Ok(decoded_str) = std::str::from_utf8(&decoded) else {
        return false;
    };
    let Some((user, pass)) = decoded_str.split_once(':') else {
        return false;
    };
    state.config.read().find_account(user, pass)
}

const STYLE_CSS: &str = r#"
:root {
  --bg: #0f1419;
  --panel: #1a2332;
  --text: #e7ecf3;
  --muted: #9aa7b8;
  --accent: #3b82f6;
  --border: #2a3545;
  --row: #151c27;
}
* { box-sizing: border-box; }
body {
  margin: 0;
  font-family: "Segoe UI", system-ui, sans-serif;
  background: var(--bg);
  color: var(--text);
}
#wrapper { max-width: 960px; margin: 0 auto; padding: 16px; }
#menu-panel {
  display: flex; justify-content: space-between; align-items: baseline;
  background: var(--panel); border: 1px solid var(--border);
  border-radius: 10px; padding: 12px 16px; margin-bottom: 12px;
}
.brand { font-weight: 700; color: var(--accent); }
.path { color: var(--muted); font-size: 14px; word-break: break-all; }
#files {
  background: var(--panel); border: 1px solid var(--border);
  border-radius: 10px; overflow: hidden;
}
.item {
  display: grid;
  grid-template-columns: 1fr 140px 90px;
  gap: 8px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--row);
}
.item:hover { background: #1c2533; }
.item a { color: var(--text); text-decoration: none; display: flex; gap: 8px; align-items: center; }
.item.dir a { color: #93c5fd; }
.item .mtime, .item .size { color: var(--muted); font-size: 13px; text-align: right; align-self: center; }
#serverinfo { margin-top: 16px; text-align: center; color: var(--muted); font-size: 12px; }
#serverinfo a { color: var(--muted); }
#upload-form {
  margin: 8px 0;
  padding: 12px 16px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 10px;
}
#upload-form input[type="file"] {
  color: var(--text);
  font-size: 13px;
  margin-right: 8px;
}
#upload-form button {
  background: var(--accent);
  color: white;
  border: none;
  padding: 6px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}"
h1 { font-size: 20px; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_full() {
        assert!(matches!(parse_range(None, 100), RangeParse::Full));
    }

    #[test]
    fn range_partial() {
        match parse_range(Some("bytes=0-9"), 100) {
            RangeParse::Partial { start: 0, end: 9 } => {}
            _ => panic!("expected partial"),
        }
        match parse_range(Some("bytes=50-"), 100) {
            RangeParse::Partial { start: 50, end: 99 } => {}
            _ => panic!("expected open end"),
        }
        match parse_range(Some("bytes=-10"), 100) {
            RangeParse::Partial { start: 90, end: 99 } => {}
            _ => panic!("expected suffix"),
        }
    }
}
