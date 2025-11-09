use std::sync::Mutex;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Coordinate {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bob {
    length_rod: f64,
    mass: f64,
    theta: f64,
    omega: f64,
    coordinate: Coordinate,
}
#[derive(Clone, Debug, PartialEq)]
struct Pendulum {
    g: f64,
    origin: Coordinate,
    bobs: Vec<Bob>,
}

#[derive(Clone, Debug, PartialEq)]

struct AppDataInner {
    pendulum: Pendulum,
}

type AppData = Mutex<AppDataInner>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {})
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
