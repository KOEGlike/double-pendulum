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
        let mut mtx = DMatrix::<f64>::zeros(n, n);

        for i in 0..n {
            let li = self.bobs[i].length_rod;
            for j in 0..n {
                let lj = self.bobs[j].length_rod;
                let theta_diff = self.bobs[i].theta - self.bobs[j].theta;
                let mut sum_m = 0.0;
                for k in std::cmp::max(i, j)..n {
                    sum_m += self.bobs[k].mass;
                }
                mtx[(i, j)] = li * lj * sum_m * theta_diff.cos();
            }
        }
        mtx
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
        let li = self.bobs[i].length_rod;
        let lj = self.bobs[j].length_rod;
        let s_ij = suffix[std::cmp::max(i, j)];
        let theta_diff = self.bobs[i].theta - self.bobs[j].theta;
        let delta = (if i == k { 1.0 } else { 0.0 }) - (if j == k { 1.0 } else { 0.0 });
        -s_ij * li * lj * theta_diff.sin() * delta
    }

    fn coriolis(&self) -> DVector<f64> {
        let n = self.n();
        let mut c = DVector::<f64>::zeros(n);
        let suffix = self.suffix_masses();

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
        let suffix = self.suffix_masses();
        let mut g_vec = DVector::<f64>::zeros(n);

        for i in 0..n {
            let li = self.bobs[i].length_rod;
            let s_i = suffix[i];
            // ∂U/∂θ_i = - l_i * sin(theta_i) * (sum_{k>=i} m_k * g)
            g_vec[i] = -li * self.bobs[i].theta.sin() * (s_i * GRAVITATIONAL_ACCELERATION);
        }
        g_vec
    }

    fn step(&mut self, dt: f64) {
        let n = self.n();
        let m = self.mass_matrix();
        let c = self.coriolis();
        let g = self.gravity();

        // Equations: M * theta_dd + C + G = 0  => theta_dd = - M^{-1} (C + G)
        let rhs = -(&c + &g);
        // solve for accelerations
        let a = match m.clone().lu().solve(&rhs) {
            Some(sol) => sol,
            None => {
                // fallback: if matrix singular, zero accelerations
                DVector::zeros(n)
            }
        };

        // symplectic Euler integrate
        for i in 0..n {
            self.bobs[i].omega += a[i] * dt;
        }
        for i in 0..n {
            self.bobs[i].theta += self.bobs[i].omega * dt;
        }

        // update coordinates (positions) — cumulative sums from root
        let mut cum_x = 0.0;
        let mut cum_y = 0.0;
        for i in 0..n {
            let xi = self.bobs[i].length_rod * self.bobs[i].theta.sin();
            let yi = self.bobs[i].length_rod * self.bobs[i].theta.cos();
            cum_x += xi;
            cum_y += yi;
            self.bobs[i].coordinate = Coordinate::new(cum_x, cum_y);
        }
    }
}

impl Default for Pendulum {
    fn default() -> Self {
        Self {
            bobs: vec![
                Bob::new(120.0, 10.0, PI / 10.0, 0.0),
                Bob::new(120.0, 20.0, PI / 10.0, 0.0),
                Bob::new(120.0, 10.0, PI / 10.0, 0.0),
                Bob::new(120.0, 10.0, PI / 10.0, 0.0),
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
        .invoke_handler(tauri::generate_handler![
            pendulum_state,
            add_bob,
            remove_bob,
            modify_bob
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BobState {
    theta: f64,
    omega: f64,
    position: Coordinate,
    mass: f64,
    length_rod: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PendulumState {
    bobs: Vec<BobState>,
}

#[tauri::command]
async fn pendulum_state(
    data: tauri::State<'_, AppData>,
    channel: Channel<PendulumState>,
) -> Result<(), String> {
    loop {
        let state = {
            let mut app_data = data.lock().map_err(|e| e.to_string())?;
            for _ in 0..2 {
                app_data.pendulum.step(0.016);
            }
            let bob_states: Vec<BobState> = app_data
                .pendulum
                .bobs
                .iter()
                .map(|bob| BobState {
                    theta: bob.theta,
                    position: bob.coordinate,
                    mass: bob.mass,
                    length_rod: bob.length_rod,
                    omega: bob.omega,
                })
                .collect();
            PendulumState { bobs: bob_states }
        };

        channel.send(state).map_err(|e| e.to_string())?;
        tokio::time::sleep(std::time::Duration::from_millis(8)).await;
    }
}

#[tauri::command]
fn add_bob(
    data: tauri::State<'_, AppData>,
    length_rod: f64,
    mass: f64,
    theta: f64,
    omega: f64,
) -> Result<(), String> {
    let mut app_data = data.lock().map_err(|e| e.to_string())?;
    app_data
        .pendulum
        .bobs
        .push(Bob::new(length_rod, mass, theta, omega));
    Ok(())
}

#[tauri::command]
fn remove_bob(data: tauri::State<'_, AppData>, index: usize) -> Result<(), String> {
    let mut app_data = data.lock().map_err(|e| e.to_string())?;
    if index >= app_data.pendulum.bobs.len() {
        return Err("Index out of bounds".into());
    }
    app_data.pendulum.bobs.remove(index);
    Ok(())
}

#[tauri::command]
fn modify_bob(
    data: tauri::State<'_, AppData>,
    index: usize,
    length: Option<f64>,
    mass: Option<f64>,
    theta: Option<f64>,
    omega: Option<f64>,
) -> Result<(), String> {
    let mut app_data = data.lock().map_err(|e| e.to_string())?;
    if index >= app_data.pendulum.bobs.len() {
        return Err("Index out of bounds".into());
    }
    let bob = app_data.pendulum.bobs.get_mut(index).unwrap();
    if let Some(l) = length {
        bob.length_rod = l;
    }
    if let Some(m) = mass {
        bob.mass = m;
    }
    if let Some(t) = theta {
        bob.theta = t;
    }
    if let Some(o) = omega {
        bob.omega = o;
    }
    Ok(())
}
