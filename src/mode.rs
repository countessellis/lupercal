use std::env::{args,Args};
use std::str::FromStr;

///////////// Mode

#[derive(Debug, Clone)]
pub(crate) enum Mode {
  Server,
  Client,
}

impl Default for Mode {
  fn default() -> Self {
    Mode::Client
  }
}

impl FromStr for Mode {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "server" => Ok(Mode::Server),
      _        => Ok(Default::default()),
    }
  }
}

impl Mode {
  pub(crate) fn mode() -> Mode {
    log::debug!("Getting mode from arguments.");
    let mut args: Args = args();
    while let Some(arg) = args.next() {
      match arg.as_str() {
        "server" => return Mode::Server,
        "client" => return Mode::Client,
        _        => {},
      }
    }
    Default::default()
  }
}

