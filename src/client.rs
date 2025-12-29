use openssl::ssl::SslMethod;
use url::Url;
use std::env::Args;
use std::env::args;
use std::net::TcpStream;
use openssl::ssl::{SslConnector,SslVerifyMode};
use std::time::Duration;
use color_eyre::{eyre::Context, Result};
use ratatui::{
  crossterm::event::{self, Event, KeyCode, KeyEventKind},
  widgets::Paragraph,
  DefaultTerminal, Frame,
};
use ratatui::text::Line;
use ratatui::style::Style;
use ratatui::style::Modifier;

use crate::config::*;
use crate::defaults::*;
use crate::display::*;
use crate::response::*;
use crate::store::*;
use crate::util;

///////////// Client

pub(crate) struct Client {
  pub(crate) config: Config,
  pub(crate) keys: Store,
}

impl Clone for Client {
  fn clone(&self) -> Self {
    Client { config: self.config.clone(), keys: self.keys.clone() }
  }
}

impl Client {
  pub(crate) fn new(config: &Config) -> Option<Client> {
    // Create store:
    match Store::new(&config) {
      Ok(store) => {
        log::info!("Key store created successfully.");
        log::info!("Starting listener...");
        Some(Client { config: config.clone(), keys: store.clone() })
      },
      Err(err)    => {
        log::error!("Failed to create store: {}",err);
        None
      },
    }
  }

  pub(crate) fn request_from_args() -> Option<String> {
    let mut args: Args = args();
    while let Some(arg) = args.next() {
      if arg.starts_with("gemini://") {
        return Some(arg);
      }
    }
    None
  }

  pub(crate) fn request(&self,request: &String) -> Option<String> {
    match Url::parse(request) {
      Ok(url) => {
        log::info!("Making request for: {}",request);
        log::debug!("URI: {:#?}",url);
        match url.host_str() {
          Some(host) => {
            match TcpStream::connect(format!("{}:{}",host,DEFAULT_LISTEN_PORT)) {
              Ok(connection) => {
                match SslConnector::builder(SslMethod::tls()) {
                  Ok(mut builder) => {
                    builder.set_verify(SslVerifyMode::NONE);
                    let connector: SslConnector = builder.build();
                    match connector.connect(host, &connection) {
                      Ok(mut tunnel) => {
                        match tunnel.ssl_write(format!("{}\r\n",request).as_bytes()) {
                          Ok(_) => {
                            let mut buffer: [u8;1024] = [0;1024];
                            match tunnel.ssl_read(&mut buffer) {
                              Ok(len) => {
                                match Response::from_bytes(&buffer.to_vec()) {
                                  Ok(response) => {
                                    log::info!("Response: {}",response);
                                    return self.display(&url,&response);
                                  },
                                  Err(err) => {
                                    log::error!("Failed to parse response from {}: {}",request,err);  
                                  },
                                }
                              },
                              Err(err) => {
                                log::error!("Failed to read response from {}: {}",request,err);
                              },
                            }
                          },
                          Err(err) => {
                            log::error!("Failed to send request to {}: {}",request,err);
                          },
                        }
                      },
                      Err(err)   => {
                        log::error!("Failed to establish SSL connection to {}:{}: {}",host,DEFAULT_LISTEN_PORT,err);
                      }
                    }
                  },
                  Err(err) => {
                    log::error!("Failed to create SSL connector for {}:{}: {}",host,DEFAULT_LISTEN_PORT,err);
                  },
                }
              },
              Err(err)   => {
                log::error!("Failed to connect to {}:{}: {}",host,DEFAULT_LISTEN_PORT,err);
              },
            };
          },
          None => {
            log::error!("Unable to determine host.");
          },
        }
      },
      Err(err) => {
        log::error!("Failed to parse {} as an url: {}",request,err);
      },
    }
    None
  }

  pub(crate) fn display(&self, source: &Url, response: &Response) -> Option<String> {
    let payload: Option<(String,String)> = response.text();
    let mut request: Option<String> = None;
    match payload {
      Some((subtype,text)) => {
        let mut terminal = ratatui::init();
        let lines: Vec<&str> = text.lines().collect();
        let (lines,links) = Self::format(subtype,lines);
        let paragraph: Paragraph = Paragraph::new(lines);
        loop {
          terminal.draw(|frame| {
            frame.render_widget(&paragraph,frame.area());
          });
          match event::poll(Duration::from_millis(250)) {
            Ok(b) => if b {
              if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                  if let KeyCode::Char(c) = key.code {
                    if let Some(digit) = c.to_digit(10) {
                      let mut link: String = links[digit as usize - 1].clone();
                      link = util::build_abs_url(&source,&link);
                      log::debug!("Link {} chosen, link is: {}",digit,link);
                      request = Some(link);
                      break;
                    } else {
                      match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Esc       => break,
                        _                  => {},
                      }
                    }
                  };
                }
              }
            },
            Err(err) => {
              log::error!("Failed to poll for event: {}",err);
              return None;
            },
          }
        }
        ratatui::restore();
      },
      None => {},
    }
    request
  }

  fn format(subtype: String, lines: Vec<&str>) -> (Vec<Line>,Vec<String>) {
    let mut formatted: Vec<Line> = Vec::new();
    let mut links: Vec<String> = Vec::new();
    for line in lines {
      match subtype.as_str() {
        "gemini" | "gmi" => {
          formatted.push(match line {
            line if line.starts_with("###") => {
              let line: &str = line.strip_prefix("###").unwrap_or(line).trim_start();
              Line::style(line.into(),Style::new().add_modifier(Modifier::ITALIC))
            },
              line if line.starts_with("##") => {
              let line: &str = line.strip_prefix("##").unwrap_or(line).trim_start();
              Line::style(line.into(),Style::new().add_modifier(Modifier::UNDERLINED))
            },
            line if line.starts_with("#") => {
              let line: &str = line.strip_prefix("#").unwrap_or(line).trim_start();
              Line::style(line.into(),Style::new().add_modifier(Modifier::BOLD))
            },
            line if line.starts_with("=>") => {
              let line: &str = line.strip_prefix("=>").unwrap_or(line).trim_start();
              let parts: Vec<&str> = line.split_whitespace().collect();
              let link: &str = parts[0];
              links.push(link.to_string());
              let text: String = format!("[{}] {}",links.len(),if parts.len() > 1 {
                parts[1..].join(" ")
              } else {
                link.to_string()
              });
              Line::style(text.into(),Style::new().add_modifier(Modifier::UNDERLINED).add_modifier(Modifier::REVERSED))
            },
            line if line.starts_with("*") => {
              let line: &str = line.strip_prefix("*").unwrap_or(line).trim_start();
              let line: String = format!("{} {}","\u{2022}",line);
              Line::raw(line)
            },
            line if line.starts_with(">") => {
              let line: &str = line.strip_prefix("###").unwrap_or(line).trim_start();
              Line::raw(line)
            },
            _   => {
              Line::raw(line)
            },
          });
        },
        _ => formatted.push(Line::raw(line)),
      }
    }
    (formatted,links)
  }
}
