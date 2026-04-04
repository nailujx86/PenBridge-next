// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;

use penbridge_tauri_lib::bridge::BridgeState;

fn get_local_ip() -> String {
    // Use the UDP socket trick (same as original PenBridge) to find the
    // outbound LAN IP. Connecting a UDP socket to a remote address makes the
    // OS pick the correct network interface without sending any data.
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|sock| {
            sock.connect("8.8.8.8:1")?;
            let addr = sock.local_addr()?;
            Ok(addr.ip().to_string())
        })
        .unwrap_or_else(|_| {
            local_ip_address::local_ip()
                .map(|ip| ip.to_string())
                .unwrap_or_else(|_| "127.0.0.1".to_string())
        })
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let ip = get_local_ip();
    let port: u16 = 30012;

    let bridge_state = Arc::new(Mutex::new(BridgeState::new(ip.clone(), port)));

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

    println!("PenBridge server running at http://{}:{}", ip, port);

    // Run Tauri app (blocks)
    penbridge_tauri_lib::run(bridge_state, socket_io);
}