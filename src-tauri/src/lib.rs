// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod bridge;
pub mod server;

use bridge::{ServerInfo, Settings, SharedBridgeState};
use socketioxide::SocketIo;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[tauri::command]
async fn get_server_info(state: tauri::State<'_, SharedBridgeState>) -> Result<ServerInfo, String> {
    let st = state.lock().await;
    Ok(st.server_info.clone())
}

#[tauri::command]
async fn get_settings(state: tauri::State<'_, SharedBridgeState>) -> Result<Settings, String> {
    let st = state.lock().await;
    Ok(st.get_settings())
}

#[tauri::command]
async fn toggle_auth(state: tauri::State<'_, SharedBridgeState>) -> Result<bool, String> {
    let mut st = state.lock().await;
    st.needs_auth = !st.needs_auth;
    Ok(st.needs_auth)
}

#[tauri::command]
async fn toggle_himetric(state: tauri::State<'_, SharedBridgeState>) -> Result<bool, String> {
    let mut st = state.lock().await;
    st.himetric = !st.himetric;
    Ok(st.himetric)
}

#[tauri::command]
async fn toggle_touch(state: tauri::State<'_, SharedBridgeState>) -> Result<bool, String> {
    let mut st = state.lock().await;
    st.touch_input = !st.touch_input;
    Ok(st.touch_input)
}

#[tauri::command]
async fn toggle_auto_fullscreen(
    state: tauri::State<'_, SharedBridgeState>,
    io: tauri::State<'_, SocketIo>,
) -> Result<bool, String> {
    let new_value = {
        let mut st = state.lock().await;
        st.auto_fullscreen = !st.auto_fullscreen;
        st.auto_fullscreen
    };

    if let Err(err) = io
        .emit(
            "autoFullscreenChanged",
            &serde_json::json!({ "enabled": new_value }),
        )
        .await
    {
        log::warn!("Failed to broadcast fullscreen setting change: {err}");
    }

    Ok(new_value)
}

#[tauri::command]
async fn exit_client_fullscreen(io: tauri::State<'_, SocketIo>) -> Result<(), String> {
    if let Err(err) = io.emit("exitfullscreen", &()).await {
        log::warn!("Failed to send exit fullscreen request: {err}");
    }

    Ok(())
}

#[tauri::command]
async fn switch_monitor(state: tauri::State<'_, SharedBridgeState>) -> Result<String, String> {
    let mut st = state.lock().await;
    Ok(st.switch_monitor_to_foreground())
}

#[tauri::command]
async fn get_pin(state: tauri::State<'_, SharedBridgeState>) -> Result<Option<u32>, String> {
    let st = state.lock().await;
    Ok(st.pin)
}

pub fn run(bridge_state: SharedBridgeState, socket_io: SocketIo) {
    tauri::Builder::default()
        .setup(|app| {
            let open_settings =
                MenuItem::with_id(app, "open_settings", "Open Settings", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_settings, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(
                    app.default_window_icon()
                        .cloned()
                        .expect("failed to load tray icon"),
                )
                .tooltip("PenBridge")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open_settings" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .manage(bridge_state)
        .manage(socket_io)
        .invoke_handler(tauri::generate_handler![
            get_server_info,
            get_settings,
            toggle_auth,
            toggle_himetric,
            toggle_touch,
            toggle_auto_fullscreen,
            exit_client_fullscreen,
            switch_monitor,
            get_pin,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
