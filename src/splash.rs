pub type Colour = nu_ansi_term::Color;

use crate::defaults::*;

///////////// Splash

pub(crate) fn version() -> String {
  format!("
{} {}.{} ({})
Copyright (C) {} {}
Licensed under the {} License
  ",APP_NAME,VERSION_ID,BUILD_ID,BUILD_TIME,COPYRIGHT,AUTHORS,LICENSE)
}

pub(crate) fn raw_splash() -> String {
  let splash: &str =  include_str!("../resources/lupercal.txt");
  format!("\n{}\n",splash)
}

pub(crate) fn splash() -> String {
  let splash: String =  raw_splash();
  format!("\n{}\n",Colour::Fixed(178).paint(splash))
}
