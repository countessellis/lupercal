
use crate::defaults::*;

///////////// Version

pub(crate) fn version() -> String {
  format!("

{} {}.{} ({})
Copyright (C) {} {}
Licensed under the {} License
  ",APP_NAME,VERSION_ID,BUILD_ID,BUILD_TIME,COPYRIGHT,AUTHORS,LICENSE)
}

