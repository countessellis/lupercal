pub type Colour = nu_ansi_term::Color;

use crate::defaults::*;

///////////// Splash

pub fn raw_splash() -> String {
  format!("\n\n{}\n",SPLASH_TXT)
}

pub fn splash() -> String {
  let splash: String =  raw_splash();
  format!("{}",Colour::Fixed(SPLASH_COLOUR).paint(splash))
}
