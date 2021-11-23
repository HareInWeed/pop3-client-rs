#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod email;
mod pop3;

use tauri::async_runtime::Mutex;

mod msg_command {
  use crate::pop3::Msg;

  #[tauri::command]
  pub fn user_msg(name: &str) -> Result<String, String> {
    let mut msg = Msg::default();
    match msg.user(name) {
      Ok(_) => Ok(msg.into_string()),
      Err(error) => Err(error.to_string()),
    }
  }

  #[tauri::command]
  pub fn pass_msg(pass: &str) -> Result<String, String> {
    let mut msg = Msg::default();
    match msg.pass(pass) {
      Ok(_) => Ok(msg.into_string()),
      Err(error) => Err(error.to_string()),
    }
  }

  #[tauri::command]
  pub fn stat_msg() -> Result<String, String> {
    let mut msg = Msg::default();
    match msg.stat() {
      Ok(_) => Ok(msg.into_string()),
      Err(error) => Err(error.to_string()),
    }
  }

  #[tauri::command]
  pub fn list_msg(id: Option<u64>) -> Result<String, String> {
    let mut msg = Msg::default();
    match msg.list(id) {
      Ok(_) => Ok(msg.into_string()),
      Err(error) => Err(error.to_string()),
    }
  }

  #[tauri::command]
  pub fn retr_msg(id: u64) -> Result<String, String> {
    let mut msg = Msg::default();
    match msg.retr(id) {
      Ok(_) => Ok(msg.into_string()),
      Err(error) => Err(error.to_string()),
    }
  }

  #[tauri::command]
  pub fn quit_msg() -> Result<String, String> {
    let mut msg = Msg::default();
    match msg.quit() {
      Ok(_) => Ok(msg.into_string()),
      Err(error) => Err(error.to_string()),
    }
  }
}

struct State {
  connection: Option<pop3::Pop3>,
}

fn main() {
  tauri::Builder::default()
    .manage(Mutex::new(State { connection: None }))
    .invoke_handler(tauri::generate_handler![
      msg_command::user_msg,
      msg_command::pass_msg,
      msg_command::stat_msg,
      msg_command::list_msg,
      msg_command::retr_msg,
      msg_command::quit_msg,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
