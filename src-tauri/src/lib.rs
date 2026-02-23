mod scanner;
mod types;

use std::sync::Mutex;
use tauri::State;
use types::MediaItem;

struct AppState {
    library: Mutex<Vec<MediaItem>>,
}

#[tauri::command]
async fn scan_library(state: State<'_, AppState>, path: String) -> Result<Vec<MediaItem>, String> {
    // Run the heavy scanning in a blocking task to avoid blocking the async runtime
    let items = tauri::async_runtime::spawn_blocking(move || {
        scanner::scan_library(&path)
    })
    .await
    .map_err(|e| e.to_string())?;

    *state.library.lock().unwrap() = items.clone();
    Ok(items)
}

#[tauri::command]
fn search(state: State<'_, AppState>, query: String) -> Result<Vec<MediaItem>, String> {
    let library = state.library.lock().unwrap();
    if query.trim().is_empty() {
        return Ok(library.clone());
    }

    let query_lower = query.to_lowercase();
    let filtered: Vec<MediaItem> = library.iter().filter(|item| {
        item.name.to_lowercase().contains(&query_lower) ||
        item.group.to_lowercase().contains(&query_lower) ||
        item.resolution.to_lowercase().contains(&query_lower) ||
        item.source.to_lowercase().contains(&query_lower) ||
        item.video_codec.to_lowercase().contains(&query_lower) ||
        item.audio_codec.to_lowercase().contains(&query_lower) ||
        item.season.as_ref().map_or(false, |s| s.to_lowercase().contains(&query_lower))
    }).cloned().collect();

    Ok(filtered)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            library: Mutex::new(Vec::new()),
        })
        .invoke_handler(tauri::generate_handler![scan_library, search])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
