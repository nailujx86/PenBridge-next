// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod bridge;
pub mod server;

use bridge::{ServerInfo, Settings, SharedBridgeState};

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
async fn switch_monitor(state: tauri::State<'_, SharedBridgeState>) -> Result<String, String> {
    let mut st = state.lock().await;
    Ok(st.switch_monitor_to_foreground())
}

#[tauri::command]
async fn get_pin(state: tauri::State<'_, SharedBridgeState>) -> Result<Option<u32>, String> {
    let st = state.lock().await;
    Ok(st.pin)
}

pub fn run(bridge_state: SharedBridgeState) {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(bridge_state)
        .invoke_handler(tauri::generate_handler![
            get_server_info,
            get_settings,
            toggle_auth,
            toggle_himetric,
            toggle_touch,
            switch_monitor,
            get_pin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
