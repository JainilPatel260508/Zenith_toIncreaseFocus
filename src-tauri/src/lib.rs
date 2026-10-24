mod db;
mod focus;
mod rooms;
mod analytics;
mod blocker;
mod rewards;
use std::sync::Mutex;
use tauri::Manager;

#[tauri::command]
async fn request(state: tauri::State<'_, Mutex<db::Database>>, request: db::Request) -> Result<serde_json::Value, String> {
    let open_resource=matches!(&request,db::Request::OpenResource {..});
    let result=state.lock().map_err(|_| "Zenith is busy. Please restart the app.".to_string())?.dispatch(request)?;
    if open_resource && result["allowed"].as_bool()==Some(true) {
        let url=result["url"].as_str().ok_or("Invalid address")?;
        let status=std::process::Command::new("/usr/bin/open").arg("--").arg(url).status().map_err(|_| "Could not open your default browser.")?;
        if !status.success(){return Err("Could not open your default browser.".into());}
    }
    Ok(result)
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
