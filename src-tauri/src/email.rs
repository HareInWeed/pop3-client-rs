use anyhow::Error;
use mailparse::{parse_mail, MailHeaderMap};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum PartType {
  Html(String),
  Text(String),
  Bin(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Email {
  subject: String,
  from: String,
  to: String,
  raw: String,
  time: String,
  parts: Vec<PartType>,
}

impl TryFrom<Vec<u8>> for Email {
  type Error = Error;

  fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
    let mut email = Self::default();
    let mail = parse_mail(&bytes[..])?;
    email.raw = String::from_utf8_lossy(&bytes[..]).to_string();
    if let Some(subject) = mail.headers.get_first_value("Subject") {
      email.subject = subject;
    }
    if let Some(from) = mail.headers.get_first_value("From") {
      email.from = from;
    }
    if let Some(to) = mail.headers.get_first_value("To") {
      email.to = to;
    }
    if let Some(time) = mail.headers.get_first_value("Date") {
      email.time = time;
    }
    email.parts = Vec::with_capacity(mail.subparts.len());
    for part in mail.subparts.iter() {
      email.parts.push(match part.ctype.mimetype.as_str() {
        "text/plain" => PartType::Text(part.get_body()?),
        "text/html" => PartType::Html(part.get_body()?),
        _ => PartType::Bin(part.get_body_raw()?),
      })
    }

    Ok(email)
  }
}
