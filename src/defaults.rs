use env_logger::TimestampPrecision;
use log::LevelFilter;

pub(crate) const APP_NAME:   &str = "Lupercal";
pub(crate) const BUILD_NAME: &str = env!("BUILD_NAME");
pub(crate) const VERSION_ID: &str = env!("CARGO_PKG_VERSION");
pub(crate) const BUILD_TIME: &str = env!("BUILD_TIME");
pub(crate) const BUILD_ID:   &str = env!("BUILD_ID");
pub(crate) const AUTHORS:    &str = env!("CARGO_PKG_AUTHORS");
pub(crate) const COPYRIGHT:  &str = "2026";
pub(crate) const LICENSE:    &str = env!("CARGO_PKG_LICENSE");


pub(crate) const DEFAULT_LOG_LEVEL: LevelFilter                                            = LevelFilter::Debug; // LevelFilter::Info;
pub(crate) const TUI_LOG_LEVEL: LevelFilter                                                = LevelFilter::Info;
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
pub(crate) const DEFAULT_PROXY_CACHE_DIR: &str  = "cache/proxy";
pub(crate) const DEFAULT_SERVER_STORE_DIR: &str = "cache/server/store/";
pub(crate) const DEFAULT_CLIENT_STORE_DIR: &str = "cache/client/store/";
pub(crate) const DEFAULT_PROXY_STORE_DIR: &str  = "cache/proxy/store/";
pub(crate) const DEFAULT_CONFIG_DIR: &str       = "config";
pub(crate) const DEFAULT_SERVER_CONFIG: &str    = "server.cfg";
pub(crate) const DEFAULT_CLIENT_CONFIG: &str    = "client.cfg";
pub(crate) const DEFAULT_PROXY_CONFIG: &str     = "proxy.cfg";
pub(crate) const DEFAULT_CONVERT_CONFIG: &str   = "convert.cfg";

pub(crate) const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "jfif", "pjpeg", "pjp", "jpe", "jif", // JPEG
    "png",                                               // PNG
    "gif",                                               // GIF
    "svg",                                               // SVG
    "webp",                                              // WebP
    "avif",                                              // AVIF
    "apng",                                              // APNG
    "ico", "cur",                                        // Icon
    "bmp", "dib",                                        // Bitmap
    "tif", "tiff",                                       // TIFF
    "heic", "heif",                                      // HEIF
    "jxl",                                               // JPEG XL
];
