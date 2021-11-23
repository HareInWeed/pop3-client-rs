use anyhow::{anyhow, Result};

use std::{io::Write, pin::Pin};

use tokio::{
  self,
  io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufStream},
  net::TcpStream,
};

use tokio_native_tls::{native_tls, TlsConnector};

mod pop3_parser {
  use anyhow::{anyhow, Result};
  use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, crlf, digit1, space0},
    combinator::{opt, recognize, value},
    sequence::{pair, tuple},
    IResult,
  };

  /// for debug only
  #[allow(unused)]
  macro_rules! probe {
    ($e:expr) => {
      |msg| dbg!($e(msg))
    };
  }

  pub fn is_ok(msg: &[u8]) -> IResult<&[u8], bool> {
    let (msg, ok) = alt((value(true, tag("+OK")), value(false, tag("-ERR"))))(msg)?;
    Ok((msg, ok))
  }

  #[test]
  fn test_is_ok() {
    assert_eq!(is_ok("+OK".as_bytes()), Ok(("".as_bytes(), true)));
    assert_eq!(is_ok("-ERR".as_bytes()), Ok(("".as_bytes(), false)));
    assert_eq!(
      is_ok("+OK some other message".as_bytes()),
      Ok((" some other message".as_bytes(), true))
    );
    assert_eq!(
      is_ok("-ERR some other message".as_bytes()),
      Ok((" some other message".as_bytes(), false))
    );
    assert!(is_ok("some random message".as_bytes()).is_err());
  }

  fn get_status_line(
    trim_ok: bool,
    trim_err: bool,
  ) -> impl Fn(&[u8]) -> IResult<&[u8], Result<&[u8], &[u8]>> {
    move |msg| {
      let (msg, (ok, status_msg, _)) = tuple((is_ok, take_until("\r\n"), crlf))(msg)?;
      let (trimmed_status_msg, _) = space0(status_msg)?;
      Ok((
        msg,
        if ok {
          Ok(if trim_ok {
            trimmed_status_msg
          } else {
            status_msg
          })
        } else {
          Err(if trim_err {
            trimmed_status_msg
          } else {
            status_msg
          })
        },
      ))
    }
  }

  pub fn parse_status_line(msg: Vec<u8>) -> Result<String> {
    // todo: eliminate allocation of String by reuse message buffer
    match get_status_line(true, true)(&msg) {
      Ok((_, Ok(msg))) => Ok(String::from_utf8_lossy(msg).to_string()),
      Ok((_, Err(msg))) => Err(anyhow! {String::from_utf8_lossy(msg).to_string()}),
      Err(err) => Err(anyhow! {err.to_string()}),
    }
  }

  #[test]
  fn test_get_status_line() {
    let get_status_line = get_status_line(true, true);

    assert!(get_status_line("+OK".as_bytes()).is_err());
    assert!(get_status_line("-ERR".as_bytes()).is_err());
    assert!(get_status_line("some random message".as_bytes()).is_err());
    assert!(get_status_line("some random message\r\n".as_bytes()).is_err());

    assert_eq!(
      get_status_line("+OK\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Ok("".as_bytes())))
    );
    assert_eq!(
      get_status_line("+OK \r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Ok("".as_bytes())))
    );
    assert_eq!(
      get_status_line("+OKsome message\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Ok("some message".as_bytes())))
    );
    assert_eq!(
      get_status_line("+OK some message\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Ok("some message".as_bytes())))
    );

    assert_eq!(
      get_status_line("-ERR\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Err("".as_bytes())))
    );
    assert_eq!(
      get_status_line("-ERR \r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Err("".as_bytes())))
    );
    assert_eq!(
      get_status_line("-ERRsome message\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Err("some message".as_bytes())))
    );
    assert_eq!(
      get_status_line("-ERR some message\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Err("some message".as_bytes())))
    );
  }

  pub fn get_line(msg: &[u8]) -> IResult<&[u8], Option<&[u8]>> {
    let (msg, (octet, line, _)) = tuple((opt(char('.')), take_until("\r\n"), crlf))(msg)?;
    if octet.is_some() && line.len() == 0 {
      Ok((msg, None))
    } else {
      Ok((msg, Some(line)))
    }
  }

  pub fn get_line_with_crlf(msg: &[u8]) -> IResult<&[u8], Option<&[u8]>> {
    let (msg, (octet, line)) =
      pair(opt(char('.')), recognize(pair(take_until("\r\n"), crlf)))(msg)?;
    if octet.is_some() && line == &b"\r\n"[..] {
      Ok((msg, None))
    } else {
      Ok((msg, Some(line)))
    }
  }

  #[test]
  fn test_get_line() {
    assert!(get_line("abcd 1234".as_bytes()).is_err());
    assert!(get_line("abcd 1234\r".as_bytes()).is_err());
    assert!(get_line("abcd 1234\n".as_bytes()).is_err());

    assert_eq!(
      get_line("abcd 1234\r\n".as_bytes()),
      Ok(("".as_bytes(), Some("abcd 1234".as_bytes())))
    );
    assert_eq!(
      get_line("abcd 1234\r\nabcd 1234".as_bytes()),
      Ok(("abcd 1234".as_bytes(), Some("abcd 1234".as_bytes())))
    );

    assert_eq!(
      get_line(".abcd 1234\r\nabcd 1234".as_bytes()),
      Ok(("abcd 1234".as_bytes(), Some("abcd 1234".as_bytes())))
    );
    assert_eq!(
      get_line("..abcd 1234\r\nabcd 1234".as_bytes()),
      Ok(("abcd 1234".as_bytes(), Some(".abcd 1234".as_bytes())))
    );
    assert_eq!(
      get_line("..\r\nabcd 1234".as_bytes()),
      Ok(("abcd 1234".as_bytes(), Some(".".as_bytes())))
    );
    assert_eq!(
      get_line("\r\nabcd 1234".as_bytes()),
      Ok(("abcd 1234".as_bytes(), Some("".as_bytes())))
    );
    assert_eq!(
      get_line(".\r\nabcd 1234".as_bytes()),
      Ok(("abcd 1234".as_bytes(), None))
    );
  }

  #[test]
  fn test_get_line_with_crlf() {
    assert!(get_line_with_crlf("abcd 1234".as_bytes()).is_err());
    assert!(get_line_with_crlf("abcd 1234\r".as_bytes()).is_err());
    assert!(get_line_with_crlf("abcd 1234\n".as_bytes()).is_err());

    assert_eq!(
      get_line_with_crlf("abcd 1234\r\n".as_bytes()),
      Ok(("".as_bytes(), Some("abcd 1234\r\n".as_bytes())))
    );
    assert_eq!(
      get_line_with_crlf("abcd 1234\r\nabcd 1234".as_bytes()),
      Ok(("abcd 1234".as_bytes(), Some("abcd 1234\r\n".as_bytes())))
    );

    assert_eq!(
      get_line_with_crlf(".abcd 1234\r\nabcd 1234".as_bytes()),
      Ok(("abcd 1234".as_bytes(), Some("abcd 1234\r\n".as_bytes())))
    );
    assert_eq!(
      get_line_with_crlf("..abcd 1234\r\nabcd 1234".as_bytes()),
      Ok(("abcd 1234".as_bytes(), Some(".abcd 1234\r\n".as_bytes())))
    );
    assert_eq!(
      get_line_with_crlf("..\r\nabcd 1234".as_bytes()),
      Ok(("abcd 1234".as_bytes(), Some(".\r\n".as_bytes())))
    );
    assert_eq!(
      get_line_with_crlf("\r\nabcd 1234".as_bytes()),
      Ok(("abcd 1234".as_bytes(), Some("\r\n".as_bytes())))
    );
    assert_eq!(
      get_line_with_crlf(".\r\nabcd 1234".as_bytes()),
      Ok(("abcd 1234".as_bytes(), None))
    );
  }

  fn get_stat_result(msg: &[u8]) -> IResult<&[u8], Result<(u64, u64, &[u8]), &[u8]>> {
    let (rest, stat) = get_status_line(false, true)(msg)?;
    match stat {
      Ok(stat) => {
        let (msg, (_, msg_num, _, maildrop_bytes)) =
          tuple((char(' '), digit1, char(' '), digit1))(stat)?;
        let msg_num = std::str::from_utf8(msg_num)
          .unwrap()
          .parse::<u64>()
          .expect("too many messages");
        let maildrop_bytes = std::str::from_utf8(maildrop_bytes)
          .unwrap()
          .parse::<u64>()
          .expect("maildrop too large");
        Ok((rest, Ok((msg_num, maildrop_bytes, msg))))
      }
      Err(msg) => Ok((rest, Err(msg))),
    }
  }

  pub fn parse_stat(msg: Vec<u8>) -> Result<(u64, u64, String)> {
    // todo: eliminate allocation of String by reuse message buffer
    match get_stat_result(&msg) {
      Ok((_, Ok((mail_num, maildrop_bytes, msg)))) => Ok((
        mail_num,
        maildrop_bytes,
        String::from_utf8_lossy(msg).to_string(),
      )),
      Ok((_, Err(msg))) => Err(anyhow! {String::from_utf8_lossy(msg).to_string()}),
      Err(err) => Err(anyhow! {err.to_string()}),
    }
  }

  #[test]
  fn test_get_stat_result() {
    assert!(get_stat_result("+OK\r\n".as_bytes()).is_err());
    assert!(get_stat_result("+OK\r\n".as_bytes()).is_err());
    assert!(get_stat_result("+OK \r\n".as_bytes()).is_err());
    assert!(get_stat_result("+OK  \r\n".as_bytes()).is_err());
    assert!(get_stat_result("+OK 123 \r\n".as_bytes()).is_err());
    assert!(get_stat_result("+OK  123 456\r\n".as_bytes()).is_err());
    assert!(get_stat_result("+OK 123  456\r\n".as_bytes()).is_err());
    assert_eq!(
      get_stat_result("+OK 123 456\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Ok((123, 456, "".as_bytes()))))
    );
    assert_eq!(
      get_stat_result("+OK 123 456 \r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Ok((123, 456, " ".as_bytes()))))
    );
    assert_eq!(
      get_stat_result("+OK 123 456 additional message\r\nrest".as_bytes()),
      Ok((
        "rest".as_bytes(),
        Ok((123, 456, " additional message".as_bytes()))
      ))
    );

    assert_eq!(
      get_stat_result("-ERR\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Err("".as_bytes())))
    );
    assert_eq!(
      get_stat_result("-ERR error message\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Err("error message".as_bytes())))
    );
  }

  fn get_scan_listing(msg: &[u8]) -> IResult<&[u8], (u64, u64)> {
    let (msg, (mail_num, _, mail_bytes)) = tuple((digit1, char(' '), digit1))(msg)?;
    let mail_num = std::str::from_utf8(mail_num)
      .unwrap()
      .parse::<u64>()
      .expect("invalid index");
    let mail_bytes = std::str::from_utf8(mail_bytes)
      .unwrap()
      .parse::<u64>()
      .expect("mail too large");
    Ok((msg, (mail_num, mail_bytes)))
  }

  fn get_single_list_result(msg: &[u8]) -> IResult<&[u8], Result<(u64, u64, &[u8]), &[u8]>> {
    let (rest, list) = get_status_line(true, true)(msg)?;
    match list {
      Ok(list) => {
        let (msg, (mail_num, mail_bytes)) = get_scan_listing(list)?;
        Ok((rest, Ok((mail_num, mail_bytes, msg))))
      }
      Err(msg) => Ok((rest, Err(msg))),
    }
  }

  fn get_list_result(msg: &[u8]) -> IResult<&[u8], Result<(Vec<(u64, u64)>, &[u8]), &[u8]>> {
    let (mut rest, list) = get_status_line(true, true)(msg)?;
    match list {
      Ok(msg) => {
        let mut list = Vec::<(u64, u64)>::new();
        loop {
          let (new_rest, scan_listing) = get_line(rest)?;
          rest = new_rest;
          if let Some(scan_listing) = scan_listing {
            let (_discarded_additional_msg, scan_listing) = get_scan_listing(scan_listing)?;
            list.push(scan_listing)
          } else {
            break Ok((rest, Ok((list, msg))));
          }
        }
      }
      Err(msg) => Ok((rest, Err(msg))),
    }
  }

  pub fn parse_single_list(msg: Vec<u8>) -> Result<(u64, u64, String)> {
    // todo: eliminate allocation of String by reuse message buffer
    match get_single_list_result(&msg) {
      Ok((_, Ok((mail_id, mail_bytes, msg)))) => Ok((
        mail_id,
        mail_bytes,
        String::from_utf8_lossy(msg).to_string(),
      )),
      Ok((_, Err(msg))) => Err(anyhow! {String::from_utf8_lossy(msg).to_string()}),
      Err(err) => Err(anyhow! {err.to_string()}),
    }
  }

  pub fn parse_list(msg: Vec<u8>) -> Result<(Vec<(u64, u64)>, String)> {
    // todo: eliminate allocation of String by reuse message buffer
    match get_list_result(&msg) {
      Ok((_, Ok((scan_listings, msg)))) => {
        Ok((scan_listings, String::from_utf8_lossy(msg).to_string()))
      }
      Ok((_, Err(msg))) => Err(anyhow! {String::from_utf8_lossy(msg).to_string()}),
      Err(err) => Err(anyhow! {err.to_string()}),
    }
  }

  #[test]
  fn test_get_single_list_result() {
    assert!(get_single_list_result("+OK\r\n".as_bytes()).is_err());
    assert!(get_single_list_result("+OK \r\n".as_bytes()).is_err());
    assert!(get_single_list_result("+OK  \r\n".as_bytes()).is_err());
    assert!(get_single_list_result("+OK 123\r\n".as_bytes()).is_err());
    assert!(get_single_list_result("+OK 123 \r\n".as_bytes()).is_err());
    assert!(get_single_list_result("+OK 123  456\r\n".as_bytes()).is_err());
    assert_eq!(
      get_single_list_result("+OK 123 456\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Ok((123, 456, "".as_bytes()))))
    );
    assert_eq!(
      get_single_list_result("+OK  123 456 \r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Ok((123, 456, " ".as_bytes()))))
    );
    assert_eq!(
      get_single_list_result("+OK 123 456 additional message\r\nrest".as_bytes()),
      Ok((
        "rest".as_bytes(),
        Ok((123, 456, " additional message".as_bytes()))
      ))
    );

    assert_eq!(
      get_single_list_result("-ERR\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Err("".as_bytes())))
    );
    assert_eq!(
      get_single_list_result("-ERR error message\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Err("error message".as_bytes())))
    );
  }

  #[test]
  fn test_get_list_result() {
    assert!(get_list_result("+OK\r\n".as_bytes()).is_err());
    assert!(get_list_result("+OK additional message".as_bytes()).is_err());
    assert!(get_list_result("+OK additional message\r\nrandom message".as_bytes()).is_err());

    assert_eq!(
      get_list_result("+OK additional message\r\n.\r\nrest".as_bytes()),
      Ok((
        "rest".as_bytes(),
        Ok((vec![], "additional message".as_bytes()))
      ))
    );

    assert_eq!(
      get_list_result("+OK additional message\r\n123 456\r\n.\r\nrest".as_bytes()),
      Ok((
        "rest".as_bytes(),
        Ok((vec![(123, 456)], "additional message".as_bytes()))
      ))
    );

    assert_eq!(
      get_list_result("+OK additional message\r\n12 34\r\n56 78\r\n.\r\nrest".as_bytes()),
      Ok((
        "rest".as_bytes(),
        Ok((vec![(12, 34), (56, 78)], "additional message".as_bytes()))
      ))
    );

    assert_eq!(
      get_single_list_result("-ERR\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Err("".as_bytes())))
    );
    assert_eq!(
      get_single_list_result("-ERR error message\r\nrest".as_bytes()),
      Ok(("rest".as_bytes(), Err("error message".as_bytes())))
    );
  }

  fn get_retr_result(msg: &[u8]) -> IResult<&[u8], Result<(Vec<u8>, &[u8]), &[u8]>> {
    let (mut rest, retr) = get_status_line(true, true)(msg)?;
    match retr {
      Ok(msg) => {
        let mut mail = Vec::<u8>::new();
        loop {
          let (new_rest, line) = get_line_with_crlf(rest)?;
          rest = new_rest;
          if let Some(line) = line {
            mail.extend_from_slice(line);
          } else {
            break Ok((rest, Ok((mail, msg))));
          }
        }
      }
      Err(msg) => Ok((rest, Err(msg))),
    }
  }

  pub fn parse_retr_result(msg: Vec<u8>) -> Result<(Vec<u8>, String)> {
    // todo: eliminate allocation of String by reuse message buffer
    match get_retr_result(&msg) {
      Ok((_, Ok((mail, ok_msg)))) => Ok((mail, String::from_utf8_lossy(ok_msg).to_string())),
      Ok((_, Err(msg))) => Err(anyhow! {String::from_utf8_lossy(msg).to_string()}),
      Err(err) => Err(anyhow! {err.to_string()}),
    }
  }
}

pub struct Msg {
  buf: Vec<u8>,
}

impl Msg {
  pub fn with_maximum_len(len: usize) -> Self {
    Self {
      buf: Vec::<u8>::with_capacity(len),
    }
  }
  pub fn get_msg(&self) -> &[u8] {
    self.buf.as_slice()
  }
  pub fn into_string(self) -> String {
    String::from_utf8(self.buf).unwrap()
  }
  pub fn user(&mut self, name: &str) -> Result<&[u8]> {
    self.buf.clear();
    write!(&mut self.buf, "USER {}\r\n", name)?;
    Ok(self.get_msg())
  }
  pub fn pass(&mut self, secret: &str) -> Result<&[u8]> {
    self.buf.clear();
    write!(&mut self.buf, "PASS {}\r\n", secret)?;
    Ok(self.get_msg())
  }
  pub fn stat(&mut self) -> Result<&[u8]> {
    self.buf.clear();
    write!(&mut self.buf, "STAT\r\n")?;
    Ok(self.get_msg())
  }
  pub fn list(&mut self, id: Option<u64>) -> Result<&[u8]> {
    self.buf.clear();
    if let Some(id) = id {
      write!(&mut self.buf, "LIST {}\r\n", id)?;
    } else {
      write!(&mut self.buf, "LIST\r\n")?;
    }
    Ok(self.get_msg())
  }
  pub fn retr(&mut self, msg: u64) -> Result<&[u8]> {
    self.buf.clear();
    write!(&mut self.buf, "RETR {}\r\n", msg)?;
    Ok(self.get_msg())
  }
  pub fn quit(&mut self) -> Result<&[u8]> {
    self.buf.clear();
    write!(&mut self.buf, "QUIT\r\n")?;
    Ok(self.get_msg())
  }
}

impl Default for Msg {
  fn default() -> Self {
    Self::with_maximum_len(27)
  }
}

trait AsyncReadWrite: AsyncRead + AsyncWrite {}
impl<T> AsyncReadWrite for T where T: AsyncRead + AsyncWrite {}

pub struct Pop3 {
  socket: BufStream<Pin<Box<dyn AsyncReadWrite + Send>>>,
  msg: Msg,
}

impl Pop3 {
  pub async fn new(addr: &str, with_tls: bool) -> Result<(Self, String)> {
    let (domain, port) = addr
      .rsplit_once(":")
      .map(|(domain, port)| (domain, port.parse::<u16>()))
      .unwrap_or((addr, Ok(if with_tls { 995 } else { 110 })));
    let port = port?;

    let mut pop3 = Self {
      socket: BufStream::new({
        let stream = TcpStream::connect((domain, port)).await?;
        if with_tls {
          let connecter: TlsConnector = native_tls::TlsConnector::new()?.into();
          Box::pin(connecter.connect(domain, stream).await?)
        } else {
          Box::pin(stream)
        }
      }),
      msg: Default::default(),
    };

    let greeting_msg = pop3_parser::parse_status_line(pop3.read_response().await?)?;

    Ok((pop3, greeting_msg))
  }

  async fn send_msg(&mut self) -> Result<()> {
    self.socket.write_all(self.msg.get_msg()).await?;
    self.socket.flush().await?;
    Ok(())
  }

  async fn read_response(&mut self) -> Result<Vec<u8>> {
    let mut buf = Vec::<u8>::new();
    while !(buf.len() >= 2 && &buf[buf.len() - 2..] == &b"\r\n"[..]) {
      if self.socket.read_until(b'\n', &mut buf).await? == 0 {
        break;
      }
    }
    Ok(buf)
  }

  async fn read_multiline_response(&mut self) -> Result<Vec<u8>> {
    let mut buf = Vec::<u8>::new();
    loop {
      if self.socket.read_until(b'\n', &mut buf).await? == 0 {
        return Ok(buf);
      }
      let len = buf.len();
      if len > 5 && &buf[len - 5..] == &b"\r\n.\r\n"[..] {
        return Ok(buf);
      }
    }
  }

  pub async fn user(&mut self, name: &str) -> Result<String> {
    self.msg.user(name)?;
    self.send_msg().await?;
    let buf = self.read_response().await?;
    pop3_parser::parse_status_line(buf)
  }

  pub async fn pass(&mut self, secret: &str) -> Result<String> {
    self.msg.pass(secret)?;
    self.send_msg().await?;
    let buf = self.read_response().await?;
    pop3_parser::parse_status_line(buf)
  }

  pub async fn stat(&mut self) -> Result<(u64, u64, String)> {
    self.msg.stat()?;
    self.send_msg().await?;
    let buf = self.read_response().await?;
    pop3_parser::parse_stat(buf)
  }

  pub async fn list(&mut self, id: Option<u64>) -> Result<(Vec<(u64, u64)>, String)> {
    self.msg.list(id)?;
    self.send_msg().await?;
    if id.is_some() {
      let buf = self.read_response().await?;
      let (id, size, msg) = pop3_parser::parse_single_list(buf)?;
      Ok((vec![(id, size)], msg))
    } else {
      let buf = self.read_multiline_response().await?;
      pop3_parser::parse_list(buf)
    }
  }

  pub async fn retr(&mut self, id: u64) -> Result<(Vec<u8>, String)> {
    self.msg.retr(id)?;
    self.send_msg().await?;
    let buf = self.read_multiline_response().await?;
    pop3_parser::parse_retr_result(buf)
  }

  pub async fn quit(mut self) -> Result<String> {
    self.msg.quit()?;
    self.send_msg().await?;
    let buf = self.read_response().await?;
    pop3_parser::parse_status_line(buf)
  }
}

#[tokio::test]
async fn test_pop3() -> Result<()> {
  use std::env::var;

  let (mut pop3, welcome_msg) = Pop3::new(&var("POP3_ADDR").unwrap(), true).await?;
  println!("{}", welcome_msg);
  pop3.user(&var("POP3_USER").unwrap()).await?;
  pop3.pass(&var("POP3_PASS").unwrap()).await?;

  let (msg_num, maildrop_bytes, additional_msg) = pop3.stat().await?;
  println!("{} {} \"{}\"", msg_num, maildrop_bytes, additional_msg);

  let (list, msg) = pop3.list(None).await?;
  println!("{:?}", msg);
  println!("{:#?}", list);

  if let Some((id, _)) = list.first().cloned() {
    println!("{:?}", pop3.list(Some(id)).await);
    let (mail, msg) = pop3.retr(id).await?;
    println!("+OK {}", msg);
    println!("{:?}", String::from_utf8(mail));
  }
  // println!("{:?}", pop3.list(Some(0)).await);
  // println!("{:?}", pop3.list(Some(std::u64::MAX)).await);

  let goodbye_message = pop3.quit().await?;
  println!("{}", goodbye_message);
  Ok(())
}
