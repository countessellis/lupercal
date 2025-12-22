use log::LevelFilter;
use env_logger::TimestampPrecision;

pub(crate) const APP_NAME:    &str = "Lupercal";
pub(crate) const VERSION_ID:  &str = env!("CARGO_PKG_VERSION");
pub(crate) const BUILD_TIME:  &str = env!("BUILD_TIME");
pub(crate) const BUILD_ID:    &str = env!("BUILD_ID");

pub(crate) const DEFAULT_LOG_LEVEL: LevelFilter                                            = LevelFilter::Debug; // LevelFilter::Info;
pub(crate) const DEFAULT_LOG_FORMAT_TIMESTAMP: Option<env_logger::fmt::TimestampPrecision> = Some(TimestampPrecision::Millis); // None;
pub(crate) const DEFAULT_LOG_FORMAT_LEVEL: bool                                            = true; // false;
pub(crate) const DEFAULT_LOG_FORMAT_TARGET: bool                                           = true; // false;
pub(crate) const DEFAULT_LOG_TARGET: env_logger::fmt::Target                               = env_logger::fmt::Target::Stdout;

pub(crate) const DEFAULT_LISTEN_ADDRESS: &str = "0.0.0.0:1965";
pub(crate) const SERVER_NAME: &str            = "lupercal.ipnhome.ironpixie.net";
