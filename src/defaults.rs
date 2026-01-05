use log::LevelFilter;
use env_logger::TimestampPrecision;

pub(crate) const APP_NAME:    &str = "Lupercal";
pub(crate) const BUILD_NAME:  &str = env!("BUILD_NAME");
pub(crate) const VERSION_ID:  &str = env!("CARGO_PKG_VERSION");
pub(crate) const BUILD_TIME:  &str = env!("BUILD_TIME");
pub(crate) const BUILD_ID:    &str = env!("BUILD_ID");

pub(crate) const DEFAULT_LOG_LEVEL: LevelFilter                                            = LevelFilter::Debug; // LevelFilter::Info;
pub(crate) const DEFAULT_LOG_FORMAT_TIMESTAMP: Option<env_logger::fmt::TimestampPrecision> = Some(TimestampPrecision::Millis); // None;
pub(crate) const DEFAULT_LOG_FORMAT_LEVEL: bool                                            = true; // false;
pub(crate) const DEFAULT_LOG_FORMAT_TARGET: bool                                           = true; // false;
pub(crate) const DEFAULT_LOG_TARGET: env_logger::fmt::Target                               = env_logger::fmt::Target::Stderr;

pub(crate) const DEFAULT_KEY_SIZE: u32          = 8192;
pub(crate) const DEFAULT_LISTEN_ADDR: &str      = "0.0.0.0";
pub(crate) const DEFAULT_LISTEN_PORT: &str      = "1965";
pub(crate) const DEFAULT_SERVER_NAME: &str      = "localhost.localdomain";
pub(crate) const DEFAULT_CLIENT_NAME: &str      = "username@localhost.localdomain";
pub(crate) const DEFAULT_CONTENT_DIR: &str      = "content";
pub(crate) const DEFAULT_SERVER_CACHE_DIR: &str = "cache/server";
pub(crate) const DEFAULT_CLIENT_CACHE_DIR: &str = "cache/client";
pub(crate) const DEFAULT_SERVER_STORE_DIR: &str = "cache/server/store/";
pub(crate) const DEFAULT_CLIENT_STORE_DIR: &str = "cache/client/store/";
pub(crate) const DEFAULT_CONFIG_DIR: &str       = "config/";
pub(crate) const DEFAULT_SERVER_CONFIG: &str    = "config/server.cfg";
pub(crate) const DEFAULT_CLIENT_CONFIG: &str    = "config/client.cfg";
