//! Socket-level check of the per-client IPC rate limit (G46, ADR 0015 and 0016).
mod common;

use std::time::Duration;
use swayve::daemon::{DaemonEvent, start_daemon_ipc_with_limits};
use swayve::ratelimit::IpcLimits;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::sync::mpsc;

async fn wait_for_socket(path: &str) {
    for _ in 0..100 {
        if UnixStream::connect(path).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("socket {path} never came up");
}

#[tokio::test]
async fn test_over_limit_lines_drop_and_client_recovers() {
    let socket_path = common::unique_path("rate", ".sock");
    let _ = fs::remove_file(socket_path).await;

    let (event_tx, mut event_rx) = mpsc::channel::<DaemonEvent>(1024);
    let listener = tokio::spawn(async move {
        start_daemon_ipc_with_limits(socket_path, event_tx, IpcLimits::default()).await
    });
    wait_for_socket(socket_path).await;

    let focus = b"{\"method\":\"daemon.focus\"}\n";
    let mut client = UnixStream::connect(socket_path).await.expect("connect");
    for _ in 0..250 {
        client.write_all(focus).await.unwrap();
    }
    client.flush().await.unwrap();

    // Count what the flood delivered. The bucket starts at 200 and refills at 100/s,
    // so the count is 200 up to about 205 (H1 in EXECUTION_PLAN_23).
    let mut delivered = 0;
    while let Ok(Some(DaemonEvent::Focus)) =
        tokio::time::timeout(Duration::from_millis(300), event_rx.recv()).await
    {
        delivered += 1;
    }
    assert!((200..=205).contains(&delivered), "delivered {delivered}, expected 200 to 205");

    // The client is still connected: poll until one more line is delivered.
    // Polling, not sleeping, to order events: the bucket refills 1 token per 10 ms.
    let mut recovered = false;
    for _ in 0..100 {
        client.write_all(focus).await.unwrap();
        client.flush().await.unwrap();
        if let Ok(Some(DaemonEvent::Focus)) =
            tokio::time::timeout(Duration::from_millis(20), event_rx.recv()).await
        {
            recovered = true;
            break;
        }
    }
    assert!(recovered, "no focus line delivered after the flood");

    listener.abort();
    let _ = fs::remove_file(socket_path).await;
}
