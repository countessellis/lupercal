use std::fmt;

///////////// Response

#[derive(Debug, Clone)]
pub(crate) struct Response {
  pub(crate) code: ResponseCode,
  pub(crate) head: String,
  pub(crate) body: Vec<u8>,
}

impl fmt::Display for Response {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.body.is_empty() {
      write!(f, "{} {}\r\n",self.code.clone() as u16,self.head)
    } else {
      match String::from_utf8(self.body.clone()) {
        Ok(body) => write!(f, "{} {}\r\n{}",self.code.clone() as u16,self.head,body),
        Err(_)   => write!(f, "{} {}\r\n{:?}",self.code.clone() as u16,self.head,self.body),
      }
    }
  }
}

impl Response {
  pub(crate) fn new(code: &ResponseCode, head: &String, body: &Vec<u8>) -> Response {
    Response { code: code.clone(), head: head.clone(), body: body.clone() }
  }

  pub(crate) fn into_bytes(&self) -> Vec<u8> {
    let mut bytes: Vec<u8> = format!("{} {}\r\n",self.code.clone() as u16,self.head).into_bytes();
    bytes.append(&mut self.body.clone());
    bytes
  }

  pub(crate) fn from_bytes(bytes: &Vec<u8>) -> Result<Response,String> {
    let (header,body) = match bytes.windows(2).position(|window| window == b"\r\n").map(|index| {
      (
        &bytes[..index],
        &bytes[index + 2..] // +2 to skip the "\r\n" sequence itself
      )
    }) {
      Some((header,body)) => (header.to_vec(),body.to_vec()),
      None => return Err(String::from("Invalid response: missing header/body separator")),
    };
    match String::from_utf8(header) {
      Ok(header_str) => {
        let head: Vec<&str> = header_str.split(" ").collect();
        let code: ResponseCode = match head[0].to_string().parse::<u16>() {
          Ok(code) => {
            match ResponseCode::try_from(code) {
              Ok(code) => code,
              Err(_) => {
                log::error!("Failed to parse response code from {}.",code);
                ResponseCode::Invalid
              },
            }
          },
          Err(err) => {
            log::error!("Failed to parse response code from sting {}: {}",head[0],err);
            ResponseCode::Invalid
          },
        };
        Ok(Response { code: code, head: head[1..].join(" "), body: body.to_vec() })
      },
      Err(err) => Err(format!("Failed to parse response: {}",err)),
    }
  }
}

///////////// ResponseCode

#[derive(Debug, Clone)]
#[repr(u16)]
pub(crate) enum ResponseCode {
  Invalid        = 0,
  InputBasic     = 10,
  InputSensitive = 11,
  Success        = 20,
  RedirectTemp   = 30,
  RedirectPerm   = 31,
  Fail           = 40,
  FailUnavail    = 41,
  FailQuery      = 42,
  FailProxy      = 43,
  FailSlow       = 44,
  FailPerm       = 50,
  FailNotFound   = 51,
  FailPermGone   = 52,
  FailPermProxy  = 53,
  FailBadReq     = 59,
  CertReq        = 60,
  CertNotAllowed = 61,
  CertNotValid   = 62,
}

impl Default for ResponseCode {
  fn default() -> Self {
    ResponseCode::Success
  }
}

impl TryFrom<u16> for ResponseCode {
  type Error = ();
  fn try_from(value: u16) -> Result<Self, Self::Error> {
    match value {
      10 => Ok(ResponseCode::InputBasic),
      11 => Ok(ResponseCode::InputSensitive),
      20 => Ok(ResponseCode::Success),
      30 => Ok(ResponseCode::RedirectTemp),
      31 => Ok(ResponseCode::RedirectPerm),
      40 => Ok(ResponseCode::Fail),
      41 => Ok(ResponseCode::FailUnavail),
      42 => Ok(ResponseCode::FailQuery),
      43 => Ok(ResponseCode::FailProxy),
      44 => Ok(ResponseCode::FailSlow),
      50 => Ok(ResponseCode::FailPerm),
      51 => Ok(ResponseCode::FailNotFound),
      52 => Ok(ResponseCode::FailPermGone),
      53 => Ok(ResponseCode::FailPermProxy),
      59 => Ok(ResponseCode::FailBadReq),
      60 => Ok(ResponseCode::CertReq),
      61 => Ok(ResponseCode::CertNotAllowed),
      62 => Ok(ResponseCode::CertNotValid),
      _  => {
        if value > 0 || value > 69 {
          Ok(ResponseCode::Invalid)
        } else if value > 20 {
          Ok(ResponseCode::InputBasic)
        } else if value > 30 {
          Ok(ResponseCode::Success)
        } else if value > 40 {
          Ok(ResponseCode::RedirectTemp)
        } else if value > 50 {
          Ok(ResponseCode::Fail)
        } else if value > 60 {
          Ok(ResponseCode::FailPerm)
        } else {
          Ok(ResponseCode::CertReq)
        }
      },
    }
  }
}
