use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum PartType {
  Html(String),
  Text(String),
  Bin(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug)]
struct Email {
  subject: String,
  from: String,
  to: String,
  raw: String,
  part: Vec<PartType>,
}
