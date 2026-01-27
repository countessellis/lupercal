use gag::Hold;
use openssl::{
  ssl::{ErrorCode,SslConnector,SslMethod,SslVerifyMode,SslVersion},
  x509::X509VerifyResult,
  pkey::PKey,
};
use ratatui::{
  crossterm::event::{self, Event, KeyCode, KeyEventKind},
  crossterm::terminal,
  prelude::*,
  text::Line,
  style::{Style,Modifier},
  widgets::*,
};
use std::{fs,io::ErrorKind,io::Write,net::TcpStream,thread,time::Duration};
use strip_prefix_suffix_sane::StripPrefixSuffixSane;
use textwrap::wrap;
use tui_logger::{TuiLoggerWidget,TuiWidgetState};
use url::Url;

use crate::config::*;
use crate::defaults::*;
use crate::mode::*;
use crate::response::*;
use crate::request::*;
use crate::store::*;

use crate::splash;
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
        if config.mode == Mode::Client { Self::splash(); }
        Some(Client { config: config.clone(), keys: store.clone() })
      },
      Err(err)    => {
        log::error!("Failed to create store: {}",err);
        None
      },
    }
  }

  pub(crate) fn request(&self,request: &Request) -> Option<Response> {
    let mut request: Request = request.clone();
    let redirect_cache: String = format!("{}/redirects.txt",self.config.cache_dir);
    match fs::read_to_string(redirect_cache) {
      Ok(redirect_cache) => {
        for entry in redirect_cache.lines() {
          let parts: Vec<String> = entry.split(" ").map(|part| part.to_string()).collect();
          if parts[0] == request.next {
            log::info!("Following cached perminant redirect from {} to {}",request.next,parts[1]);
            request.next = parts[1].clone();
            break;
          }
        }
      },
      Err(err) => if err.kind() != ErrorKind::NotFound {
        log::error!("Failed to load redirect cache, skipping: {}",err);
      },
    }
    match request.as_url() {
      Some(url) => {
        log::info!("Making request for: {}",request.next);
        match url.host_str().map(|host| host.to_string()) {
          Some(mut host) => {
            match self.config.mode {
              Mode::Proxy => {
                if !self.config.allow.is_empty() && !self.config.allow.contains(&host.to_string()) {
                  return Some(Response::new(&ResponseCode::FailPerm,&String::from("Host Not Allowed"),&Vec::new(),&None))
                }
                if !self.config.deny.is_empty() && self.config.deny.contains(&host.to_string()) {
                  return Some(Response::new(&ResponseCode::FailPerm,&String::from("Host Not Allowed"),&Vec::new(),&None))
                }
              },
              _ => {
                if !self.config.proxy.is_empty() {
                  host = self.config.proxy.clone();
                };
                if !self.config.allow.is_empty() && !self.config.allow.contains(&host.to_string()) { return None }
                if !self.config.deny.is_empty() && self.config.deny.contains(&host.to_string()) { return None }
              },
            }
            match TcpStream::connect(format!("{}:{}",host,DEFAULT_LISTEN_PORT)) {
              Ok(connection) => {
                match SslConnector::builder(SslMethod::tls()) {
                  Ok(mut builder) => {
                    builder.set_verify(SslVerifyMode::PEER);
                    match PKey::from_rsa(self.keys.keys.keypair.clone()) {
                      Ok(pkey) => {
                        log::debug!("Initializing client certificate for connection.");
                        if let Err(err) = builder.set_certificate(&self.keys.keys.cert) {
                          log::error!("Failed to set certificate: {}",err);
                        } else if let Err(err) = builder.set_private_key(&pkey) {
                          log::error!("Failed to set private: {}",err);
                        } else {
                          log::debug!("Certificate and key set.");
                        }
                        if let Err(err) = builder.set_min_proto_version(Some(SslVersion::TLS1_2)) {
                          log::error!("Failed to set minimum TLS version to 1.2: {}",err);
                        }
                      },
                      Err(err) => {
                        log::error!("Failed to prepair key, proceeding without client certificate: {}",err);
                      },
                    }
                    let cache = self.keys.clone();
                    let remote = host.clone();
                    builder.set_verify_callback(SslVerifyMode::PEER, move |_preverify_ok, context| {
                      if context.error_depth() == 0 {
                        let mut cache = cache.clone();
                        let cert = context.current_cert();
                        match cert {
                          Some(cert) => {
                            if cache.verify(&remote,&cert.to_owned()) {
                              context.set_error(openssl::x509::X509VerifyResult::OK);
                              return true;
                            } else {
                              context.set_error(X509VerifyResult::APPLICATION_VERIFICATION);
                              return false;
                            }
                          },
                          None => {
                            log::error!("No cert provided by {}.",remote);
                            context.set_error(X509VerifyResult::APPLICATION_VERIFICATION);
                            return false;
                          },
                        }
                      }
                      true
                    });
                    let connector: SslConnector = builder.build();
                    match connector.connect(&host, &connection) {
                      Ok(mut tunnel) => {
                        match tunnel.ssl_write(&request.as_bytes()) {
                          Ok(_) => {

                            let mut buffer: [u8;16384] = [0;16384];
                            let mut bytes: Vec<u8> = Vec::new();
                            loop {
                              match tunnel.ssl_read(&mut buffer) {
                                Ok(0) => {
                                  match Response::from_bytes(&Some(request.clone()),&bytes) {
                                    Ok(response) => {
                                      log::debug!("All bytes returned: {}",bytes.len());
                                      return Some(response);
                                    },
                                    Err(err) => {
                                      log::error!("Failed to parse response from {}: {}",request.next,err);  
                                    },
                                  }
                                },
                                Ok(len) => {
                                  log::debug!("Received {} bytes...",len);
                                  bytes.extend_from_slice(&buffer[..len]);
                                },
                                Err(err) => {
                                  if err.code() == ErrorCode::ZERO_RETURN {
                                    match Response::from_bytes(&Some(request.clone()),&bytes) {
                                      Ok(response) => {
                                        log::debug!("All bytes returned: {}",bytes.len());
                                        return Some(response);
                                      },
                                      Err(err) => {
                                        log::error!("Failed to parse response from {}: {}",request.next,err);  
                                      },
                                    }
                                  }
                                  if let Some(io_err) = err.io_error() {
                                    if io_err.kind() == std::io::ErrorKind::Interrupted {
                                      continue;
                                    }
                                  }
                                  log::error!("Failed to read response from {}: {}",request.next,err);
                                  break;
                                },
                              }
                            }
                          },
                          Err(err) => {
                            log::error!("Failed to send request to {}: {}",request.next,err);
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
      None => {
        log::error!("Failed to parse {} as an url",request.next);
      },
    }
    None
  }

  pub(crate) fn splash() {
    match terminal::enable_raw_mode() {
      Ok(()) =>  {
        let mut terminal = ratatui::init();
        match terminal.draw(|frame| {
          let full_area = frame.area();
          let splash: Text = Text::from(splash::raw_splash()).fg(Color::Rgb(215,175,0));
          let paragraph = Paragraph::new(splash).alignment(Alignment::Center);
          frame.render_widget(paragraph,full_area);
        }) {
          Ok(_) => thread::sleep(Duration::from_secs(2)),
          Err(err) => log::error!("Failed to display splash screen: {}",err),
        }
        ratatui::restore();
      },
      Err(_) => {},
    }
  }

  pub(crate) fn tui(&self, response: &Response) {
    let mut request: Option<Request>;
    let mut response: Response = response.clone();
    let mut terminal = ratatui::init();
    let hold: Option<Hold> = match Hold::stderr() {
      Ok(hold) => Some(hold),
      Err(_)    => None,
    };
    let mut vert_scroll_pos: usize = 0;
    let mut hori_scroll_pos: usize = 0;
    let mut line_count: usize = 0;
    let mut link_count: usize = 0;
    let mut page_len: usize = 0;
    let mut page_wid: usize = 0;
    let mut max_width: usize = 0;
    let mut vert_scroll_state: ScrollbarState  = Default::default();
    let mut hori_scroll_state: ScrollbarState  = Default::default();
    let logger_state = TuiWidgetState::new().set_default_display_level(TUI_LOG_LEVEL);
    let mut lines: Vec<Line> = Vec::new();
    let mut links: Vec<String> = Vec::new();
    'main: loop {
      match response.request {
        Some(ref source) => {
          match source.as_url() {
            Some(url) => {
              let payload: Option<(String,String)> = match &response.code {
                ResponseCode::Success => {
                  response.text()
                },
                ResponseCode::RedirectTemp => {
                  if source.redirects < 5 {
                    request = Some(Request::new(&self.config,&response.head,&Some(source.clone()),source.redirects.saturating_add(1)));
                    match request {
                      Some(ref request) => {
                        match self.request(&request) {
                          Some(next) => {
                            response = next;
                            continue 'main;
                          },
                          None => {
                            let body: String = format!("#Redirect failed:\n\n> {} {}",response.code.clone() as u16,response.head);
                            Some((String::from("gemini"),body))
                          },
                        }
                      },
                      None => {
                        let body: String = format!("#Redirect failed:\n\n> {} {}",response.code.clone() as u16,response.head);
                        Some((String::from("gemini"),body))
                      },
                    }
                  } else {
                    let body: String = format!("#Too many redirects:\n\n> {} {}",response.code.clone() as u16,response.head);
                    Some((String::from("gemini"),body))
                  }
                },
                ResponseCode::RedirectPerm => {
                  if source.redirects < 5 {
                    let redirect_cache: String = format!("{}/redirects.txt",self.config.cache_dir);
                    match fs::OpenOptions::new().write(true).append(true).create(true).open(redirect_cache) {
                      Ok(mut redirect_cache) => {
                        match writeln!(redirect_cache,"{} {}",source.next,response.head) {
                          Ok(()) => {},
                          Err(err) => log::error!("Failed to cache perminant redirect, treating as temporary: {}",err),
                        }
                      },
                      Err(err) => log::error!("Failed to open redirect cache, treating as temporary: {}",err),
                    }
                    request = Some(Request::new(&self.config,&response.head,&Some(source.clone()),source.redirects.saturating_add(1)));
                    match request {
                      Some(ref request) => {
                        match self.request(&request) {
                          Some(next) => {
                            response = next;
                            continue 'main;
                          },
                          None => {
                            let body: String = format!("#Redirect failed:\n\n> {} {}",response.code.clone() as u16,response.head);
                            Some((String::from("gemini"),body))
                          },
                        }
                      },
                      None => {
                        let body: String = format!("#Redirect failed:\n\n> {} {}",response.code.clone() as u16,response.head);
                        Some((String::from("gemini"),body))
                      },
                    }
                  } else {
                    let body: String = format!("#Too many redirects:\n\n> {} {}",response.code.clone() as u16,response.head);
                    Some((String::from("gemini"),body))
                  }
                },
                _ => {
                  let body: String = format!("#Unsuccessful request:\n\n> {} {}",response.code.clone() as u16,response.head);
                  Some((String::from("gemini"),body))
                },
              };
              match payload {
                Some((subtype,text)) => {
                  match terminal::enable_raw_mode() {
                    Ok(()) => {},
                    Err(_) => return,
                  }
                  let block = Block::bordered()
                    .title(Line::from(format!(" {} [{}] ",APP_NAME,source.next)).centered())
                    .title_bottom(Line::from("q to quit, number for link").centered())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().add_modifier(Modifier::BOLD))
                    .padding(Padding::new(1,1,1,1));
                  match terminal.draw(|frame| {
                    let chunks = Layout::vertical([Constraint::Min(0),Constraint::Length(26)]).split(frame.area());
                    let main_area = chunks[0];
                    let log_area = chunks[1];
                    (lines,links) = Self::format(subtype.clone(),text.clone(),(main_area.width-4) as usize);
                    line_count = lines.len();
                    link_count = links.len();
                    max_width = lines.iter().map(|line| line.width()).max().unwrap_or(0);
                    let paragraph: Paragraph = Paragraph::new(lines.clone()).scroll((vert_scroll_pos as u16,hori_scroll_pos as u16)).block(block);
                    page_len = main_area.height as usize - 1;
                    page_wid = main_area.width as usize - 1;
                    vert_scroll_state = vert_scroll_state.content_length(line_count.saturating_add(6).saturating_sub(page_len)).position(vert_scroll_pos);
                    hori_scroll_state = hori_scroll_state.content_length(max_width.saturating_add(13).saturating_sub(page_wid)).position(hori_scroll_pos);
                    frame.render_widget(&paragraph,main_area);
                    frame.render_stateful_widget(
                      Scrollbar::new(ScrollbarOrientation::VerticalRight),
                      main_area.inner(Margin { vertical: 1, horizontal: 0 }),
                      &mut vert_scroll_state,
                    );
                    frame.render_stateful_widget(
                      Scrollbar::new(ScrollbarOrientation::HorizontalBottom),
                      main_area.inner(Margin { vertical: 0, horizontal: 1 }),
                      &mut hori_scroll_state,
                    );
                    let log_widget = TuiLoggerWidget::default()
                      .block(Block::bordered().title("Log"))
                      .output_target(false)
                      .output_timestamp(None)
                      .output_level(None)
                      .output_file(false)
                      .output_line(false)
                      .style(Style::default().fg(Color::Blue))
                      .state(&logger_state);
                    frame.render_widget(log_widget, log_area);
                  }) {
                    Ok(_) => {},
                    Err(err) => {
                      log::error!("Failed to display response: {}",err);
                      break;
                    },
                  }
                  let max_vert_scroll: usize = if line_count+2 < page_len { 0 } else { line_count+4-page_len };
                  let max_hori_scroll: usize = if max_width.saturating_add(2) < page_wid { 0 } else { max_width.saturating_sub(page_wid).saturating_add(10) };
                  tui_logger::move_events();
                  match event::poll(Duration::from_millis(250)) {
                    Ok(b) => if b {
                      if let Ok(Event::Key(key)) = event::read() {
                        if key.kind == KeyEventKind::Press {
                          match key.code {
                            KeyCode::Char(c) if c.to_digit(10).is_some() => {
                              let mut link_number: Vec<char> = Vec::new();
                              link_number.push(c);
                              loop {
                                match event::poll(Duration::from_millis(250)) {
                                  Ok(b) => if b {
                                    if let Ok(Event::Key(key)) = event::read() {
                                      if key.kind == KeyEventKind::Press {
                                        match key.code {
                                          KeyCode::Char(c) if c.to_digit(10).is_some() => {
                                            link_number.push(c);
                                          },
                                          KeyCode::Enter => {
                                            let link_number: String = link_number.into_iter().collect();
                                            if let Ok(index) = link_number.parse::<usize>() {
                                              if index <= link_count {
                                                let mut link: String = links[index - 1].clone();
                                                link = util::build_abs_url(&url,&link);
                                                log::info!("Link {} chosen, link is: {}",c,link);
                                                match Url::parse(&link) {
                                                  Ok(url) => {
                                                    match url.scheme() {
                                                      "gemini" => {
                                                        request = Some(Request::new(&self.config,&link,&Some(source.clone()),0));
                                                        match request {
                                                          Some(ref request) => {
                                                            match self.request(&request) {
                                                              Some(next) => {
                                                                response = next;
                                                                continue 'main;
                                                              },
                                                              None => break,
                                                            }
                                                          },
                                                          None => break,
                                                        }
                                                      },
                                                      _ => {
                                                        log::error!("Using {} protocol is not supported, only gemini is.",url.scheme());
                                                        break;
                                                      },
                                                    }
                                                  },
                                                  Err(_) => break,
                                                }
                                              }
                                            }
                                            break;
                                          },
                                          _ => break,
                                        }
                                      }
                                    }
                                  },
                                  Err(_) => break,
                                }
                              }
                            },
                            KeyCode::Esc | KeyCode::Char('q') => break,
                            KeyCode::Backspace | KeyCode::Char('p') => {
                              request = source.prev();
                              match request {
                                Some(ref request) => {
                                  match self.request(&request) {
                                    Some(next) => {
                                      response = next;
                                      continue 'main;
                                    },
                                    None => break,
                                  }
                                },
                                None => break,
                              }
                            },
                            KeyCode::Enter | KeyCode::Down | KeyCode::Char('j') => {
                              vert_scroll_pos = vert_scroll_pos.saturating_add(1).min(max_vert_scroll);
                              vert_scroll_state = vert_scroll_state.position(vert_scroll_pos);
                            },
                            KeyCode::Up | KeyCode::Char('k') => {
                              vert_scroll_pos = vert_scroll_pos.saturating_sub(1);
                              vert_scroll_state = vert_scroll_state.position(vert_scroll_pos);
                            },
                            KeyCode::Char(' ') | KeyCode::PageDown | KeyCode::Char('v') => {
                              vert_scroll_pos = vert_scroll_pos.saturating_add(page_len).min(max_vert_scroll);
                              vert_scroll_state = vert_scroll_state.position(vert_scroll_pos);
                            },
                            KeyCode::PageUp | KeyCode::Char('b') => {
                              vert_scroll_pos = vert_scroll_pos.saturating_sub(page_len);
                              vert_scroll_state = vert_scroll_state.position(vert_scroll_pos);
                            },
                            KeyCode::Home | KeyCode::Char('g') => {
                              vert_scroll_pos = 0;
                              vert_scroll_state = vert_scroll_state.position(vert_scroll_pos);
                            },
                            KeyCode::End | KeyCode::Char('G') => {
                              vert_scroll_pos = max_vert_scroll;
                              vert_scroll_state = vert_scroll_state.position(vert_scroll_pos);
                            },
                            KeyCode::Right => {
                              hori_scroll_pos = hori_scroll_pos.saturating_add(1).min(max_hori_scroll);
                              hori_scroll_state = hori_scroll_state.position(hori_scroll_pos);
                            },
                            KeyCode::Left => {
                              hori_scroll_pos = hori_scroll_pos.saturating_sub(1);
                              hori_scroll_state = hori_scroll_state.position(hori_scroll_pos);
                            },
                            KeyCode::Char('s') => {
                              match response.save(&self.config) {
                                Ok(path) => log::info!("Saved to {}",path),
                                Err(err) => log::error!("Failed to save to file: {}",err),
                              }
                            },
                            _ => {},
                          }
                        }
                      }
                    },
                    Err(err) => {
                      log::error!("Failed to poll for event: {}",err);
                      return;
                    },
                  }
                },
                None => {
                  match response.datatype() {
                    Some(_) => {
                    log::info!("Received non-text file, saving to cache.");
                        match response.save(&self.config) {
                        Ok(path) => log::info!("Saved to {}",path),
                        Err(err) => log::error!("Failed to save to file: {}",err),
                      }
                    },
                    None => {},
                  }
                },
              }
            },
            None => {
              log::error!("Failed to convert {} to a URL.",source.next);
              return;
            },
          }
        },
        None => return,
      }
    }
    ratatui::restore();
    match hold {
      Some(hold) => drop(hold),
      None       => {},
    }
  }

  fn format(subtype: String, text: String, width: usize) -> (Vec<Line<'static>>,Vec<String>) {
    let unformatted: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let mut formatted: Vec<Line> = Vec::new();
    let mut links: Vec<String> = Vec::new();
    let mut raw: bool = false;
    for line in unformatted {
      match subtype.as_str() {
        "gemini" | "gmi" => {
          if raw {
            if line.starts_with("```") {
              raw ^= true;
            } else {
              formatted.push(Line::raw(line));
            }
          } else {
            match line {
              line if line.starts_with("###") => {
                let line: String = line.strip_prefix("###").unwrap_or(&line).strip_prefix_sane(" ").to_string();
                let lines: Vec<String> = wrap(&line,width).iter().map(|s| s.to_string()).collect();
                for line in lines {
                  formatted.push(Line::style(line.into(),Style::new().italic()));
                }
              },
                line if line.starts_with("##") => {
                let line: String = line.strip_prefix("##").unwrap_or(&line).strip_prefix_sane(" ").to_string();
                let lines: Vec<String> = wrap(&line,width).iter().map(|s| s.to_string()).collect();
                for line in lines {
                  formatted.push(Line::style(line.into(),Style::new().underlined()));
                }
              },
              line if line.starts_with("#") => {
                let line: String = line.strip_prefix("#").unwrap_or(&line).strip_prefix_sane(" ").to_string();
                let lines: Vec<String> = wrap(&line,width).iter().map(|s| s.to_string()).collect();
                for line in lines {
                  formatted.push(Line::style(line.into(),Style::new().bold()));
                }
              },
              line if line.starts_with(">") => {
                let line: String = line.strip_prefix(">").unwrap_or(&line).strip_prefix_sane(" ").to_string();
                let lines: Vec<String> = wrap(&line,width-2).iter().map(|s| format!("  {}",s)).collect();
                for line in lines {
                  formatted.push(Line::style(line.into(),Style::new().italic().dim()));
                }
              },
              line if line.starts_with("=>") => {
                let line: String = line.strip_prefix("=>").unwrap_or(&line).strip_prefix_sane(" ").to_string();
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
                let line: String = line.strip_prefix("*").unwrap_or(&line).strip_prefix_sane(" ").to_string();
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
                let lines: Vec<String> = wrap(&line,width).iter().map(|s| s.to_string()).collect();
                for line in lines {
                  formatted.push(Line::raw(line));
                }
              },
            };
          }
        },
        _ => {
          let lines: Vec<String> = wrap(&line,width).iter().map(|s| s.to_string()).collect();
          for line in lines {
            formatted.push(Line::raw(line));
          }
        },
      }
    }
    (formatted,links)
  }
}
