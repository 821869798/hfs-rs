//! Smoke test: start server, list dir, download file with range.

use std::{fs, sync::Arc, time::Duration};

use hfs_rs::server::AppState;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn download_and_range() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("hello.txt");
    fs::write(&file, b"Hello HFS-RS range-test!!").unwrap();

    let state = AppState::new();
    {
        let mut cfg = state.config.write();
        cfg.bind = "127.0.0.1".into();
        cfg.port = 0; // will rebind below via direct serve after patching listen
    }

    // Bind port 0 manually by temporarily overriding through serve path:
    // We call http::serve after setting a free port discovered here.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    state.config.write().port = port;

    {
        let mut vfs = state.vfs.write();
        let root = vfs.root_id();
        vfs.add_file(root, &file).unwrap();
    }

    let token = CancellationToken::new();
    let state_c = Arc::clone(&state);
    let token_c = token.clone();
    let server = tokio::spawn(async move {
        hfs_rs::http::serve(state_c, token_c).await.unwrap();
    });

    // wait until running
    for _ in 0..50 {
        if state.server.is_running() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(state.server.is_running());

    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{port}");

    let list = client.get(format!("{base}/")).send().await.unwrap();
    assert!(list.status().is_success());
    let body = list.text().await.unwrap();
    assert!(body.contains("hello.txt"));

    let full = client
        .get(format!("{base}/hello.txt"))
        .send()
        .await
        .unwrap();
    assert!(full.status().is_success());
    assert_eq!(
        full.bytes().await.unwrap().as_ref(),
        b"Hello HFS-RS range-test!!"
    );

    let partial = client
        .get(format!("{base}/hello.txt"))
        .header("Range", "bytes=0-4")
        .send()
        .await
        .unwrap();
    assert_eq!(partial.status(), reqwest::StatusCode::PARTIAL_CONTENT);
    assert_eq!(partial.bytes().await.unwrap().as_ref(), b"Hello");

    token.cancel();
    let _ = server.await;
}
