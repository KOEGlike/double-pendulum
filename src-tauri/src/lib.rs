use nalgebra::{DMatrix, DVector};
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
    bobs: Vec<Bob>,
}

impl Pendulum {
    fn new(bobs: Vec<Bob>) -> Self {
        Self { bobs }
    }

    fn n(&self) -> usize {
        self.bobs.len()
    }

    fn mass_matrix(&self) -> DMatrix<f64> {
        let n = self.n();
        let mut mass_matrix = DMatrix::<f64>::zeros(n, n);

        for i in 0..n {
            for j in 0..n {
                let mut sum = 0.0;
                for k in std::cmp::max(i, j)..n {
                    sum += self.bobs[k].mass
                        * self.bobs[i].length_rod
                        * self.bobs[j].length_rod
                        * (self.bobs[i].theta - self.bobs[j].theta).cos();
                }
                mass_matrix[(i, j)] = sum;
            }
        }

        mass_matrix
    }

    fn coriolis(&self) -> DVector<f64> {
        let n = self.n();
        let mut c = DVector::zeros(n);
        for i in 0..n {
            let mut val = 0.0;
            for j in 0..n {
                for k in std::cmp::max(i, j)..n {
                    val += self.bobs[k].mass
                        * self.bobs[i].length_rod
                        * self.bobs[j].length_rod
                        * (self.bobs[i].theta - self.bobs[j].theta).sin()
                        * self.bobs[j].omega
                        * self.bobs[i].omega;
                }
            }
            c[i] = val;
        }
        c
    }

    fn gravity(&self) -> DVector<f64> {
        let n = self.n();
        let mut g_vec = DVector::zeros(n);
        for i in 0..n {
            let mut val = 0.0;
            for k in i..n {
                val += self.bobs[k].mass
                    * GRAVITATIONAL_ACCELERATION
                    * self.bobs[i].length_rod
                    * self.bobs[i].theta.sin();
            }
            g_vec[i] = val;
        }
        g_vec
    }

    fn step(&mut self, dt: f64) {
        let n = self.n();
        let m = self.mass_matrix();
        let c = self.coriolis();
        let g = self.gravity();

        let rhs = -(c + g);
        let a = m.clone().lu().solve(&rhs).unwrap_or(DVector::zeros(n));

        for i in 0..n {
            self.bobs[i].omega += a[i] * dt;
            self.bobs[i].theta += self.bobs[i].omega * dt;
        }
    }
}

impl Default for Pendulum {
    fn default() -> Self {
        Self {
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
