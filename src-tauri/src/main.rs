// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axum::routing::get;
use socketioxide::{
    extract::SocketRef,
    SocketIo,
};

#[tokio::main]
async fn main() {
    let (layer, io) = SocketIo::new_layer();

    // Register a handler for the default namespace
    io.ns("/", async |s: SocketRef| {
        // For each "message" event received, send a "message-back" event with the "Hello World!" event
        s.on("message", async |s: SocketRef| {
            s.emit("message-back", "Hello World!").ok();
        });
    });

    let app = axum::Router::new()
    .route("/", get(async || "Hello, World!"))
    .layer(layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:30012").await.unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    penbridge_tauri_lib::run();
}