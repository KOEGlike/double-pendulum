use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};
use std::{f64::consts::PI, sync::Mutex};

use tauri::{ipc::Channel, Manager};

const GRAVITATIONAL_ACCELERATION: f64 = 9.81;

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
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

    fn suffix_masses(&self) -> Vec<f64> {
        let n = self.n();
        let mut s = vec![0.0; n];
        let mut acc = 0.0;
        for i in (0..n).rev() {
            acc += self.bobs[i].mass;
            s[i] = acc;
        }
        s
    }

    fn d_mass_matrix_dtheta(&self, i: usize, j: usize, k: usize, suffix: &[f64]) -> f64 {
        let s_ij = suffix[usize::max(i, j)];
        let li = self.bobs[i].length_rod;
        let lj = self.bobs[j].length_rod;
        let d_ik = if i == k { 1.0 } else { 0.0 };
        let d_jk = if j == k { 1.0 } else { 0.0 };
        let theta_ij = self.bobs[i].theta - self.bobs[j].theta;
        // d/dθ_k cos(θ_i - θ_j) = -sin(θ_i - θ_j) * (δ_{ik} - δ_{jk})
        -s_ij * li * lj * theta_ij.sin() * (d_ik - d_jk)
    }

    fn coriolis(&self) -> DVector<f64> {
        let n = self.n();
        let mut c = DVector::zeros(n);
        let suffix = self.suffix_masses();

        // Christoffel symbols Γ_{i j k} = 1/2 (∂M_{i k}/∂θ_j + ∂M_{i j}/∂θ_k - ∂M_{j k}/∂θ_i)
        for i in 0..n {
            let mut ci = 0.0;
            for j in 0..n {
                for k in 0..n {
                    let dM_ik_dth_j = self.d_mass_matrix_dtheta(i, k, j, &suffix);
                    let dM_ij_dth_k = self.d_mass_matrix_dtheta(i, j, k, &suffix);
                    let dM_jk_dth_i = self.d_mass_matrix_dtheta(j, k, i, &suffix);
                    let gamma = 0.5 * (dM_ik_dth_j + dM_ij_dth_k - dM_jk_dth_i);
                    ci += gamma * self.bobs[j].omega * self.bobs[k].omega;
                }
            }
            c[i] = ci;
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

        for i in 0..n {
            let x = self.bobs[i].length_rod * self.bobs[i].theta.sin()
                + if i == 0 {
                    0.0
                } else {
                    self.bobs[i - 1].coordinate.x
                };
            let y = self.bobs[i].length_rod * self.bobs[i].theta.cos()
                + if i == 0 {
                    0.0
                } else {
                    self.bobs[i - 1].coordinate.y
                };
            self.bobs[i].coordinate = Coordinate::new(x, y);
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
        .invoke_handler(tauri::generate_handler![pendulum_state])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Serialize, Deserialize)]
struct PendulumState {
    angles: Vec<f64>,
    positions: Vec<Coordinate>,
}

#[tauri::command]
async fn pendulum_state(
    data: tauri::State<'_, AppData>,
    channel: Channel<PendulumState>,
) -> Result<(), String> {
    loop {
        let state = {
            let mut app_data = data.lock().map_err(|e| e.to_string())?;
            app_data.pendulum.step(0.016);
            let angles: Vec<f64> = app_data.pendulum.bobs.iter().map(|bob| bob.theta).collect();
            let positions: Vec<Coordinate> = app_data
                .pendulum
                .bobs
                .iter()
                .map(|bob| bob.coordinate)
                .collect();
            PendulumState { angles, positions }
        };

        channel.send(state).map_err(|e| e.to_string())?;
        tokio::time::sleep(std::time::Duration::from_millis(16)).await;
    }
}
