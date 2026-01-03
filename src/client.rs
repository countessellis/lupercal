use openssl::ssl::SslMethod;
use url::Url;
use std::env::Args;
use std::env::args;
use std::net::TcpStream;
use openssl::ssl::{SslConnector,SslVerifyMode};
use std::time::Duration;
use ratatui::{
  crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
  crossterm::terminal,
  widgets::*,
  DefaultTerminal, Frame,
};
use ratatui::text::Line;
use ratatui::style::Style;
use ratatui::style::Modifier;
use ratatui::prelude::*;
use ratatui::layout::{Constraint, Flex, Rect};
use std::thread;
use textwrap::wrap;

use crate::config::*;
use crate::defaults::*;
use crate::response::*;
use crate::splash;
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
        Self::splash();
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
                            let mut buffer: [u8;16384] = [0;16384];
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

  pub(crate) fn splash() {
    match terminal::enable_raw_mode() {
      Ok(()) =>  {
        let mut terminal = ratatui::init();
        terminal.draw(|frame| {
          let full_area = frame.area();
          let splash: Text = Text::from(splash::raw_splash()).fg(Color::Rgb(215,175,0));
          let height: u16 = splash.height() as u16;
          let width: u16 = splash.width() as u16;
          let centred_area = full_area.centered_horizontally(Constraint::Length(width)).centered_vertically(Constraint::Length(height));
          let paragraph = Paragraph::new(splash).alignment(Alignment::Center);
          frame.render_widget(paragraph, full_area);
        });
        thread::sleep(Duration::from_secs(2));
        ratatui::restore();
      },
      Err(_) => {},
    }
  }

  pub(crate) fn display(&self, source: &Url, response: &Response) -> Option<String> {
    let payload: Option<(String,String)> = response.text();
    let mut request: Option<String> = None;
    match payload {
      Some((subtype,text)) => {
        match terminal::enable_raw_mode() {
          Ok(()) => {},
          Err(_) => return None,
        }
        let mut terminal = ratatui::init();
        let mut scroll_pos: usize = 0;
        let mut line_count: usize = 0;
        let mut link_count: usize = 0;
        let mut page_len: usize = 0;
        let mut scrollbar_state: ScrollbarState  = Default::default();
        let mut lines: Vec<Line> = Vec::new();
        let mut links: Vec<String> = Vec::new();
        loop {
          let block = Block::bordered()
            .title(Line::from(format!(" {} [{}] ",APP_NAME,source)).centered())
            .title_bottom(Line::from("q to quit, number for link").centered())
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().add_modifier(Modifier::BOLD))
            .padding(Padding::new(1,1,1,1));
          terminal.draw(|frame| {
            let unformatted: Vec<&str> = text.lines().collect();
            (lines,links) = Self::format(subtype.clone(),unformatted,(frame.area().width-4) as usize);
            line_count = lines.len();
            link_count = links.len();
            let paragraph: Paragraph = Paragraph::new(lines.clone()).scroll((scroll_pos as u16,0)).block(block);
            scrollbar_state.content_length(line_count).position(scroll_pos);
            page_len = frame.area().height as usize - 1;
            frame.render_widget(&paragraph,frame.area());
            frame.render_stateful_widget(
              Scrollbar::new(ScrollbarOrientation::VerticalRight),
              frame.area().inner(Margin { vertical: 1, horizontal: 0 }),
              &mut scrollbar_state,
            );
          });
          let max_scroll: usize = if line_count+2 < page_len { 0 } else { line_count+2-page_len };
          match event::poll(Duration::from_millis(250)) {
            Ok(b) => if b {
              if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                  match key.code {
                    KeyCode::Char(c) if c.to_digit(10).is_some() => {
                      let index: usize = c.to_digit(10).unwrap() as usize;
                      log::debug!("Length: {}, Index: {}",link_count,index);
                      if index < link_count {
                        let mut link: String = links[index - 1].clone();
                        link = util::build_abs_url(&source,&link);
                        log::debug!("Link {} chosen, link is: {}",c,link);
                        request = Some(link);
                        break;
                      }
                    },
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Enter | KeyCode::Down | KeyCode::Char('j') => {
                      scroll_pos = scroll_pos.saturating_add(1).min(max_scroll);
                      scrollbar_state.position(scroll_pos);
                    },
                    KeyCode::Up | KeyCode::Char('k') => {
                      scroll_pos = scroll_pos.saturating_sub(1);
                      scrollbar_state.position(scroll_pos);
                    },
                    KeyCode::Char(' ') | KeyCode::PageDown | KeyCode::Char('v') => {
                      scroll_pos = scroll_pos.saturating_add(page_len).min(max_scroll);
                      scrollbar_state.position(scroll_pos);
                    },
                    KeyCode::PageUp | KeyCode::Char('b') => {
                      scroll_pos = scroll_pos.saturating_sub(page_len);
                      scrollbar_state.position(scroll_pos);
                    },
                    KeyCode::Home | KeyCode::Char('g') => {
                      scroll_pos = 0;
                      scrollbar_state.position(scroll_pos);
                    },
                    KeyCode::End | KeyCode::Char('G') => {
                      scroll_pos = max_scroll;
                      scrollbar_state.position(scroll_pos);
                    },
                    _ => {},
                  }
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

  fn format(subtype: String, lines: Vec<&str>, width: usize) -> (Vec<Line>,Vec<String>) {
    let mut formatted: Vec<Line> = Vec::new();
    let mut links: Vec<String> = Vec::new();
    let mut raw: bool = false;
    for line in lines {
      match subtype.as_str() {
        "gemini" | "gmi" => {
          if raw {
            if line.starts_with("```") {
              raw ^= true;
            }
            formatted.push(Line::raw(line));
          } else {
            match line {
              line if line.starts_with("###") => {
                let line: &str = line.strip_prefix("###").unwrap_or(line).trim_start();
                let lines: Vec<String> = wrap(line,width).iter().map(|s| s.to_string()).collect();
                for line in lines {
                  formatted.push(Line::style(line.into(),Style::new().italic()));
                }
              },
                line if line.starts_with("##") => {
                let line: &str = line.strip_prefix("##").unwrap_or(line).trim_start();
                let lines: Vec<String> = wrap(line,width).iter().map(|s| s.to_string()).collect();
                for line in lines {
                  formatted.push(Line::style(line.into(),Style::new().underlined()));
                }
              },
              line if line.starts_with("#") => {
                let line: &str = line.strip_prefix("#").unwrap_or(line).trim_start();
                let lines: Vec<String> = wrap(line,width).iter().map(|s| s.to_string()).collect();
                for line in lines {
                  formatted.push(Line::style(line.into(),Style::new().bold()));
                }
              },
              line if line.starts_with(">") => {
                let line: &str = line.strip_prefix(">").unwrap_or(line).trim_start();
                let lines: Vec<String> = wrap(line,width-2).iter().map(|s| format!("  {}",s)).collect();
                for line in lines {
                  formatted.push(Line::style(line.into(),Style::new().italic().dim()));
                }
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
                let lines: Vec<String> = wrap(&text,width).iter().map(|s| s.to_string()).collect();
                for line in lines {
                  formatted.push(Line::style(line.into(),Style::new().underlined().reversed()));
                }
              },
              line if line.starts_with("*") => {
                let line: &str = line.strip_prefix("*").unwrap_or(line).trim_start();
                let line: String = format!("{} {}","\u{2022}",line);
                let lines: Vec<String> = wrap(&line,width).iter().map(|s| s.to_string()).collect();
                for line in lines {
                  formatted.push(Line::raw(line));
                }
              },
              line if line.starts_with("```") => {
                raw ^= true;
              },
              _   => {
                let lines: Vec<String> = wrap(line,width).iter().map(|s| s.to_string()).collect();
                for line in lines {
                  formatted.push(Line::raw(line));
                }
              },
            };
          }
        },
        _ => {
          let lines: Vec<String> = wrap(line,width).iter().map(|s| s.to_string()).collect();
          for line in lines {
            formatted.push(Line::raw(line));
          }
        },
      }
    }
    (formatted,links)
  }
}
