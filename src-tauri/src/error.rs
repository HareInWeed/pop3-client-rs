use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorMsg {
  msg: String,
}

impl<T: Display> From<T> for ErrorMsg {
  fn from(err: T) -> Self {
    Self {
      msg: err.to_string(),
    }
  }
}
