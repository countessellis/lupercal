use ansi_term::Colour;

use crate::defaults::*;

///////////// Splash

pub(crate) fn version() -> String {
  format!("\n{} {}.{} ({})",APP_NAME,VERSION_ID,BUILD_ID,BUILD_TIME)
}

pub(crate) fn raw_splash() -> String {
  let splash: &str =  include_str!("../resources/lupercal.txt");
  format!("\n{}\n",splash)
}

pub(crate) fn splash() -> String {
  let splash: String =  raw_splash();
  format!("\n{}\n",Colour::Fixed(178).paint(splash))
}
