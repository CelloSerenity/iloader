#[macro_use]
mod account;
#[macro_use]
mod device;

use crate::{
    account::{
        delete_account, get_developer_session, invalidate_account, logged_in_as, login_email_pass,
        login_stored_pass,
    },
    device::{get_provider, list_devices, set_selected_device, DeviceInfoMutex},
};
use isideload::{sideload::sideload_app, SideloadConfiguration};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
async fn sideload(
    handle: AppHandle,
    device_state: State<'_, DeviceInfoMutex>,
    app_path: String,
) -> Result<(), String> {
    let device = {
        let device_lock = device_state.lock().unwrap();
        match &*device_lock {
            Some(d) => d.clone(),
            None => return Err("No device selected".to_string()),
        }
    };

    let provider = get_provider(&device).await?;

    let config = SideloadConfiguration::default()
        .set_machine_name("iloader".to_string())
        .set_store_dir(
            handle
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {:?}", e))?,
        );

    let dev_session = get_developer_session().await.map_err(|e| e.to_string())?;

    sideload_app(&provider, &dev_session, app_path.into(), config)
        .await
        .map_err(|e| format!("Failed to sideload app: {:?}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            app.manage(DeviceInfoMutex::new(None));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            login_email_pass,
            invalidate_account,
            logged_in_as,
            login_stored_pass,
            delete_account,
            list_devices,
            sideload,
            set_selected_device,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
