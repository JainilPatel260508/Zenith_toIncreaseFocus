mod db;
mod focus;
mod rooms;
use std::sync::Mutex;
use tauri::Manager;

#[tauri::command]
async fn request(state: tauri::State<'_, Mutex<db::Database>>, request: db::Request) -> Result<serde_json::Value, String> {
    state.lock().map_err(|_| "Zenith is busy. Please restart the app.".to_string())?.dispatch(request)
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir=std::env::var_os("ZENITH_DATA_DIR").map(std::path::PathBuf::from).unwrap_or(app.path().app_data_dir()?);
            std::fs::create_dir_all(&dir)?;
            #[cfg(unix)] {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
            }
            let db=db::Database::open(&dir.join("zenith.sqlite3")).map_err(std::io::Error::other)?;
            app.manage(Mutex::new(db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![request])
        .run(tauri::generate_context!())
        .expect("Could not start Zenith");
}
