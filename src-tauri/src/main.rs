// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;

use penbridge_tauri_lib::bridge::BridgeState;

#[tokio::main]
async fn main() {
    env_logger::init();

    let port: u16 = 30012;

    let bridge_state = Arc::new(Mutex::new(BridgeState::new(port)));

    // Start socket.io / web server in background
    let server_state = bridge_state.clone();
    let (server_app, socket_io) = penbridge_tauri_lib::server::create_router(server_state);
    tokio::spawn(async move {
        penbridge_tauri_lib::server::serve_router(server_app, port).await;
    });

    // Rotate PIN every 30 seconds when authentication is enabled
    let pin_state = bridge_state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            let mut st = pin_state.lock().await;
            if st.needs_auth {
                let pin = st.generate_pin();
                log::info!("PIN rotated: {}", pin);
            }
        }
    });

    // Recheck IP every 5 seconds for network changes
    let ip_state = bridge_state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let mut st = ip_state.lock().await;
            st.update_ip();
        }
    });

    {
        let st = bridge_state.lock().await;
        println!("PenBridge server running at {}", st.server_info.url);
    }

    // Run Tauri app (blocks)
    penbridge_tauri_lib::run(bridge_state, socket_io);
}