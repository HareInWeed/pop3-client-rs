#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod email;
mod error;
mod pop3;

use anyhow::Result;
use tauri::async_runtime::Mutex;

use crate::{email::Email, error::ErrorMsg, pop3::Pop3};

mod msg_command {
  use crate::{error::ErrorMsg, pop3::Msg};

  #[tauri::command]
  pub fn user_msg(name: &str) -> Result<String, ErrorMsg> {
    let mut msg = Msg::default();
    msg.user(name)?;
    Ok(msg.into_string())
  }

  #[tauri::command]
  pub fn pass_msg(secret: &str) -> Result<String, ErrorMsg> {
    let mut msg = Msg::default();
    msg.pass(secret)?;
    Ok(msg.into_string())
  }

  #[tauri::command]
  pub fn stat_msg() -> Result<String, ErrorMsg> {
    let mut msg = Msg::default();
    msg.stat()?;
    Ok(msg.into_string())
  }

  #[tauri::command]
  pub fn list_msg(id: Option<u64>) -> Result<String, ErrorMsg> {
    let mut msg = Msg::default();
    msg.list(id)?;
    Ok(msg.into_string())
  }

  #[tauri::command]
  pub fn retr_msg(id: u64) -> Result<String, ErrorMsg> {
    let mut msg = Msg::default();
    msg.retr(id)?;
    Ok(msg.into_string())
  }

  #[tauri::command]
  pub fn quit_msg() -> Result<String, ErrorMsg> {
    let mut msg = Msg::default();
    msg.quit()?;
    Ok(msg.into_string())
  }
}

#[derive(Default)]
struct State {
  connection: Option<Pop3>,
  addr: String,
  name: String,
  pass: String,
}

#[tauri::command]
async fn connect(
  state: tauri::State<'_, Mutex<State>>,
  addr: String,
  with_tls: bool,
) -> Result<String, ErrorMsg> {
  let mut state = state.lock().await;
  if let Some(connection) = state.connection.take() {
    let _ = connection.quit().await;
  }

  let (pop3, welcome_msg) = Pop3::new(addr.as_str(), with_tls).await?;
  state.connection = Some(pop3);
  state.addr = addr;

  Ok(welcome_msg)
}

#[tauri::command]
async fn user(state: tauri::State<'_, Mutex<State>>, name: String) -> Result<String, ErrorMsg> {
  let mut state = state.lock().await;

  let connection = state
    .connection
    .as_mut()
    .ok_or("no pop3 server connection")?;

  Ok(connection.user(&name).await?)
}

#[tauri::command]
async fn pass(state: tauri::State<'_, Mutex<State>>, secret: String) -> Result<String, ErrorMsg> {
  let mut state = state.lock().await;

  let connection = state
    .connection
    .as_mut()
    .ok_or("no pop3 server connection")?;

  Ok(connection.pass(&secret).await?)
}

#[tauri::command]
async fn stat(state: tauri::State<'_, Mutex<State>>) -> Result<(u64, u64, String), ErrorMsg> {
  let mut state = state.lock().await;

  let connection = state
    .connection
    .as_mut()
    .ok_or("no pop3 server connection")?;

  Ok(connection.stat().await?)
}

#[tauri::command]
async fn list(
  state: tauri::State<'_, Mutex<State>>,
  id: Option<u64>,
) -> Result<(Vec<(u64, u64)>, String), ErrorMsg> {
  let mut state = state.lock().await;

  let connection = state
    .connection
    .as_mut()
    .ok_or("no pop3 server connection")?;

  Ok(connection.list(id).await?)
}

#[tauri::command]
async fn retr(state: tauri::State<'_, Mutex<State>>, id: u64) -> Result<(Email, String), ErrorMsg> {
  let mut state = state.lock().await;

  let connection = state
    .connection
    .as_mut()
    .ok_or("no pop3 server connection")?;

  let (raw_email, msg) = connection.retr(id).await?;

  Ok((raw_email.try_into()?, msg))
}

#[tauri::command]
async fn quit(state: tauri::State<'_, Mutex<State>>) -> Result<String, ErrorMsg> {
  let mut state = state.lock().await;

  let connection = state.connection.take().ok_or("no pop3 server connection")?;

  let msg = connection.quit().await?;

  Ok(msg)
}

fn main() {
  tauri::Builder::default()
    .manage(Mutex::new(State::default()))
    .invoke_handler(tauri::generate_handler![
      connect,
      user,
      pass,
      stat,
      list,
      retr,
      quit,
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
