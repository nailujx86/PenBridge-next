use axum::http::header;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use socketioxide::extract::{Data, SocketRef};
use socketioxide::socket::DisconnectReason;
use socketioxide::SocketIo;
use tower_http::cors::CorsLayer;

use crate::bridge::SharedBridgeState;

const WEB_CLIENT_HTML: &str = include_str!("web_client.html");

const SOCKET_IO_CLIENT_JS: &str = include_str!("vendor/socket.io.min.js");

async fn serve_client() -> Html<&'static str> {
    Html(WEB_CLIENT_HTML)
}

async fn serve_socket_io_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        SOCKET_IO_CLIENT_JS,
    )
}

pub fn create_router(state: SharedBridgeState) -> (axum::Router, SocketIo) {
    let (layer, io) = SocketIo::new_layer();

    let bridge_state = state.clone();
    io.ns("/", move |s: SocketRef| {
        let state = bridge_state.clone();
        async move {
            log::info!("Client connected: {}", s.id);

            // Pen down
            {
                let state = state.clone();
                s.on("down", move |_s: SocketRef, Data::<serde_json::Value>(data)| {
                    let state = state.clone();
                    async move {
                        let st = state.lock().await;
                        if !st.is_authenticated(&_s.id.to_string()) { return; }
                        let x = data["x"].as_f64().unwrap_or(0.0);
                        let y = data["y"].as_f64().unwrap_or(0.0);
                        let pressure = data["pressure"].as_f64().unwrap_or(0.0);
                        let erase = data["erase"].as_bool().unwrap_or(false);
                        let barrel = data["barrel"].as_bool().unwrap_or(false);
                        st.inject_pen_down(x, y, pressure, erase, barrel);
                    }
                });
            }

            // Pen up
            {
                let state = state.clone();
                s.on("up", move |_s: SocketRef, Data::<serde_json::Value>(data)| {
                    let state = state.clone();
                    async move {
                        let st = state.lock().await;
                        if !st.is_authenticated(&_s.id.to_string()) { return; }
                        let x = data["x"].as_f64().unwrap_or(0.0);
                        let y = data["y"].as_f64().unwrap_or(0.0);
                        st.inject_pen_up(x, y);
                    }
                });
            }

            // Pen hover
            {
                let state = state.clone();
                s.on("hover", move |_s: SocketRef, Data::<serde_json::Value>(data)| {
                    let state = state.clone();
                    async move {
                        let st = state.lock().await;
                        if !st.is_authenticated(&_s.id.to_string()) { return; }
                        let x = data["x"].as_f64().unwrap_or(0.0);
                        let y = data["y"].as_f64().unwrap_or(0.0);
                        let erase = data["erase"].as_bool().unwrap_or(false);
                        let barrel = data["barrel"].as_bool().unwrap_or(false);
                        st.inject_pen_hover(x, y, erase, barrel);
                    }
                });
            }

            // Pen contact (continuous movement)
            {
                let state = state.clone();
                s.on("contact", move |_s: SocketRef, Data::<serde_json::Value>(data)| {
                    let state = state.clone();
                    async move {
                        let st = state.lock().await;
                        if !st.is_authenticated(&_s.id.to_string()) { return; }
                        let x = data["x"].as_f64().unwrap_or(0.0);
                        let y = data["y"].as_f64().unwrap_or(0.0);
                        let pressure = data["pressure"].as_f64().unwrap_or(0.0);
                        let erase = data["erase"].as_bool().unwrap_or(false);
                        let barrel = data["barrel"].as_bool().unwrap_or(false);
                        let tilt_x = data["tX"].as_f64().unwrap_or(0.0);
                        let tilt_y = data["tY"].as_f64().unwrap_or(0.0);
                        st.inject_pen_contact(x, y, pressure, erase, barrel, tilt_x, tilt_y);
                    }
                });
            }

            // Touch down
            {
                let state = state.clone();
                s.on("tdown", move |_s: SocketRef, Data::<serde_json::Value>(data)| {
                    let state = state.clone();
                    async move {
                        let mut st = state.lock().await;
                        if !st.is_authenticated(&_s.id.to_string()) { return; }
                        let id = data["id"].as_u64().unwrap_or(0) as u32;
                        let x = data["x"].as_f64().unwrap_or(0.0);
                        let y = data["y"].as_f64().unwrap_or(0.0);
                        st.inject_touch_down(id, x, y);
                    }
                });
            }

            // Touch up
            {
                let state = state.clone();
                s.on("tup", move |_s: SocketRef, Data::<serde_json::Value>(data)| {
                    let state = state.clone();
                    async move {
                        let mut st = state.lock().await;
                        if !st.is_authenticated(&_s.id.to_string()) { return; }
                        let id = data["id"].as_u64().unwrap_or(0) as u32;
                        let x = data["x"].as_f64().unwrap_or(0.0);
                        let y = data["y"].as_f64().unwrap_or(0.0);
                        st.inject_touch_up(id, x, y);
                    }
                });
            }

            // Touch contact
            {
                let state = state.clone();
                s.on("tcontact", move |_s: SocketRef, Data::<serde_json::Value>(data)| {
                    let state = state.clone();
                    async move {
                        let mut st = state.lock().await;
                        if !st.is_authenticated(&_s.id.to_string()) { return; }
                        let id = data["id"].as_u64().unwrap_or(0) as u32;
                        let x = data["x"].as_f64().unwrap_or(0.0);
                        let y = data["y"].as_f64().unwrap_or(0.0);
                        st.inject_touch_contact(id, x, y);
                    }
                });
            }

            // Client size
            {
                let state = state.clone();
                s.on("clientSize", move |_s: SocketRef, Data::<serde_json::Value>(data)| {
                    let state = state.clone();
                    async move {
                        let mut st = state.lock().await;
                        if !st.is_authenticated(&_s.id.to_string()) { return; }
                        if let Some(width) = data["width"].as_f64() {
                            st.set_client_width(width);
                            log::info!("Client size change. New factor: {}", st.factor_x);
                        }
                    }
                });
            }

            // Check authentication
            {
                let state = state.clone();
                s.on("checkauthentication", move |s: SocketRef| {
                    let state = state.clone();
                    async move {
                        let mut st = state.lock().await;
                        if !st.needs_auth {
                            s.emit("authenticated", &()).ok();
                        } else {
                            s.emit("needsauthentication", &()).ok();
                            let pin = st.generate_pin();
                            log::info!("PIN for connecting client: {}", pin);
                        }
                    }
                });
            }

            // Authenticate
            {
                let state = state.clone();
                s.on("authenticate", move |s: SocketRef, Data::<serde_json::Value>(data)| {
                    let state = state.clone();
                    async move {
                        let mut st = state.lock().await;
                        let client_pin = data.as_str()
                            .and_then(|s| s.parse::<u32>().ok())
                            .or_else(|| data.as_u64().map(|n| n as u32));

                        match (client_pin, st.pin) {
                            (Some(cp), Some(sp)) if cp == sp => {
                                let sid = s.id.to_string();
                                st.add_authenticated(&sid);
                                s.emit("authenticated", &()).ok();
                                log::info!("Client {} authenticated", sid);
                            }
                            _ => {
                                log::info!("Wrong PIN from client {}", s.id);
                                let _new_pin = st.generate_pin();
                                log::info!("New PIN: {}", _new_pin);
                                s.emit("needsauthentication", &()).ok();
                            }
                        }
                    }
                });
            }

            // Ping (keep-alive)
            s.on("ping", |_s: SocketRef| async {});

            // Disconnect
            {
                let state = state.clone();
                s.on_disconnect(move |s: SocketRef, _reason: DisconnectReason| {
                    let state = state.clone();
                    async move {
                        let mut st = state.lock().await;
                        st.remove_authenticated(&s.id.to_string());
                        log::info!("Client disconnected: {}", s.id);
                    }
                });
            }
        }
    });

    let app = axum::Router::new()
        .route("/", get(serve_client))
        .route("/socket.io.min.js", get(serve_socket_io_js))
        .layer(layer)
        .layer(CorsLayer::permissive());

    (app, io)
}

pub async fn start_server(state: SharedBridgeState) {
    let port = state.lock().await.server_info.port;
    let (app, _io) = create_router(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    log::info!("Server listening on {}", addr);

    axum::serve(listener, app).await.unwrap();
}
