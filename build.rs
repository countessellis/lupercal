use std::time::SystemTime;
use chrono::{DateTime, Utc};

fn main() {
  let build_name = env!("CARGO_PKG_NAME");
  println!("cargo:rustc-env=BUILD_NAME={}",build_name);
  let build_time: DateTime<Utc> = SystemTime::now().into();
  println!("cargo:rustc-env=BUILD_TIME={}",build_time.to_rfc3339());
  let build_id: i64 = build_time.timestamp_millis();
  println!("cargo:rustc-env=BUILD_ID={}",build_id);
}
