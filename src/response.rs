use std::str::FromStr;
use std::fmt;

///////////// Response

#[derive(Debug, Clone)]
pub(crate) struct Response {
  pub(crate) code: ResponseCode,
  pub(crate) head: String,
  pub(crate) body: String,
}

impl FromStr for Response {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut body: Vec<&str> = s.lines().collect();
    let header: &str = body.remove(0);
    let mut head: Vec<&str> = header.split(" ").collect();
    let code: ResponseCode = match head.remove(0).to_string().parse::<u16>() {
      Ok(code) => {
        match ResponseCode::try_from(code) {
          Ok(code) => code,
          Err(err) => ResponseCode::Invalid,
        }
      },
      Err(err) => ResponseCode::Invalid,
    };
    Ok(Response { code: code, head: head.join(" "), body: body.join("\n") })
  }
}

impl fmt::Display for Response {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} {}\r\n{}",self.code.clone() as u16,self.head,self.body)
  }
}

impl Response {
  pub(crate) fn new(code: &ResponseCode, head: &String, body: &String) -> Response {
    Response { code: code.clone(), head: head.clone(), body: body.clone() }
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
