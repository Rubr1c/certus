use std::sync::atomic::Ordering;

use tokio::net::TcpListener;

use crate::{connection::pool, middleware::forwarding, upstream::server};

#[tokio::test]
async fn borrow_increments_counters() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let upstream = server::UpstreamServer::new(addr, 10, Default::default());

    let accept = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        stream
    });

    let conn = pool::borrow_connection(&upstream, 5).await;
    let _stream = accept.await.unwrap();

    assert!(conn.is_ok());
    assert_eq!(upstream.active_connctions.load(Ordering::Acquire), 1);
    assert_eq!(upstream.pool.total_connections.load(Ordering::Acquire), 1);
}

#[tokio::test]
async fn release_reusable_keeps_total() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let upstream = server::UpstreamServer::new(addr, 10, Default::default());

    let accept = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        stream
    });

    let conn = pool::borrow_connection(&upstream, 5).await.unwrap();
    let _stream = accept.await.unwrap();

    pool::release_connection(&upstream, conn, true).await;

    assert_eq!(upstream.active_connctions.load(Ordering::Acquire), 0);
    assert_eq!(upstream.pool.total_connections.load(Ordering::Acquire), 1);
    assert!(!upstream.pool.idle_connections.is_empty());
}

#[tokio::test]
async fn release_not_reusable_decrements_total() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let upstream = server::UpstreamServer::new(addr, 10, Default::default());

    let accept = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        stream
    });

    let conn = pool::borrow_connection(&upstream, 5).await.unwrap();
    let _stream = accept.await.unwrap();

    pool::release_connection(&upstream, conn, false).await;

    assert_eq!(upstream.active_connctions.load(Ordering::Acquire), 0);
    assert_eq!(upstream.pool.total_connections.load(Ordering::Acquire), 0);
    assert!(upstream.pool.idle_connections.is_empty());
}

#[tokio::test]
async fn borrow_returns_overloaded_at_max() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let upstream = server::UpstreamServer::new(addr, 1, Default::default());

    let accept = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        stream
    });

    let _first = pool::borrow_connection(&upstream, 5).await.unwrap();
    let _stream = accept.await.unwrap();

    let second = pool::borrow_connection(&upstream, 5).await;

    assert!(second.is_err());
}

#[tokio::test]
async fn handle_request_failure_cleans_up_counters() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let upstream = server::UpstreamServer::new(addr, 10, Default::default());

    let accept = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        drop(stream);
    });

    let req = hyper::Request::builder()
        .uri("/test")
        .body(axum::body::Body::empty())
        .unwrap();

    let res = forwarding::handle_request(&upstream, req, 5).await;
    accept.await.unwrap();

    assert!(res.is_err());
    assert_eq!(upstream.active_connctions.load(Ordering::Acquire), 0);
    assert_eq!(upstream.pool.total_connections.load(Ordering::Acquire), 0);
}
