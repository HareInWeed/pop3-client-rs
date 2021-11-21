#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::async_runtime::RwLock;

struct Counter {
  count: i32,
}

#[tauri::command]
async fn increase(state: tauri::State<'_, RwLock<Counter>>) -> Result<(), String> {
  let mut state = state.write().await;
  state.count += 1;
  Ok(())
}

#[tauri::command]
async fn decrease(state: tauri::State<'_, RwLock<Counter>>) -> Result<(), String> {
  let mut state = state.write().await;
  state.count -= 1;
  Ok(())
}

#[tauri::command]
async fn get_counter(state: tauri::State<'_, RwLock<Counter>>) -> Result<i32, String> {
  let state = state.read().await;
  Ok(state.count)
}

fn main() {
  tauri::Builder::default()
    .manage(RwLock::new(Counter { count: 0 }))
    .invoke_handler(tauri::generate_handler![increase, decrease, get_counter])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
