use std::{f64::consts::PI, sync::Mutex};

use tauri::Manager;

const GRAVITATIONAL_ACCELERATION: f64 = 9.81;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
struct Coordinate {
    x: f64,
    y: f64,
}

impl Coordinate {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bob {
    length_rod: f64,
    mass: f64,
    theta: f64,
    omega: f64,
    coordinate: Coordinate,
}

impl Bob {
    fn new(length_rod: f64, mass: f64, theta: f64, omega: f64) -> Self {
        Self {
            length_rod,
            mass,
            theta,
            omega,
            coordinate: Coordinate::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Pendulum {
    origin: Coordinate,
    bobs: Vec<Bob>,
}

impl Pendulum {
    fn new(origin: Coordinate, bobs: Vec<Bob>) -> Self {
        Self { origin, bobs }
    }
}

impl Default for Pendulum {
    fn default() -> Self {
        Self {
            origin: Coordinate::new(300.0, 100.0),
            bobs: vec![
                Bob::new(120.0, 10.0, PI / 2.0, 0.0),
                Bob::new(120.0, 10.0, PI / 2.0, 0.0),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct AppDataInner {
    pendulum: Pendulum,
}

type AppData = Mutex<AppDataInner>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(AppDataInner {
                pendulum: Pendulum::default(),
            }));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
