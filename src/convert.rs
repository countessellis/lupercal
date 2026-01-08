use std::fs;
use std::path::Path;
use std::io::BufReader;
use std::fs::File;
use std::io::BufRead;
use std::ffi::OsStr;
use fs_extra::dir::CopyOptions;
use fs_extra::copy_items;

use crate::config::*;

///////////// Convert

pub(crate) struct Convert {
  config: Config,
}

impl Clone for Convert {
  fn clone(&self) -> Self {
    Convert { config: self.config.clone() }
  }
}

impl Convert {
  pub(crate) fn new(config: &Config) -> Convert {
    Convert { config: config.clone() }
  }

  pub(crate) fn convert(&self) -> Result<(),String> {
    let input = Path::new(&self.config.convert_in);
    for entry in match fs::read_dir(input) { Ok(entry) => entry, Err(err) => return Err(format!("Failed to read directory {}: {}",self.config.convert_in,err)) } {
      match entry {
        Ok(entry) => {
          let path = entry.path();
          match path.extension().and_then(OsStr::to_str) {
            Some("gemini") | Some("gmi") => {
              match File::open(&path) {
                Ok(file) => {
                  let reader = BufReader::new(file);
                  let mut output: Vec<String> = Vec::new();
                  let mut raw: bool = false;
                  let mut list: Vec<String> = Vec::new();
                  let mut header: String = String::new();
                  for line in reader.lines() {
                    match line {
                      Ok(line) => {
                        if raw {
                          if line.starts_with("```") {
                            output.push(String::from("</PRE>"));
                            raw ^= true;
                          } else {
                            output.push(line);
                          }
                        } else if !list.is_empty() && line.starts_with("*") {
                          let line: &str = line.strip_prefix("*").unwrap_or(&line).trim_start();
                          list.push(format!("<LI>{}</LI>",line))
                        } else {
                          if !list.is_empty() {
                            output.push(String::from("<UL>"));
                            for item in list.iter() {
                              output.push(item.to_string());
                            }
                            output.push(String::from("</UL>"));
                            list.clear();
                          }
                          match line {
                            line if line.starts_with("###") => {
                              let line: &str = line.strip_prefix("###").unwrap_or(&line).trim_start();
                              output.push(format!("<H3>{}</H3>",line))
                            },
                            line if line.starts_with("##") => {
                              let line: &str = line.strip_prefix("##").unwrap_or(&line).trim_start();
                              output.push(format!("<H2>{}</H2>",line))
                            },
                            line if line.starts_with("#") => {
                              let line: &str = line.strip_prefix("#").unwrap_or(&line).trim_start();
                              if header.is_empty() { header = format!("<TITLE>{}</TITLE>",line) }
                              output.push(format!("<H1>{}</H1>",line))
                            },
                            line if line.starts_with(">") => {
                              let line: &str = line.strip_prefix(">").unwrap_or(&line).trim_start();
                              output.push(format!("<BLOCKQUOTE>{}</BLOCKQUOTE>",line))
                            },
                            line if line.starts_with("=>") => {
                              let line: &str = line.strip_prefix("=>").unwrap_or(&line).trim_start();
                              let parts: Vec<&str> = line.split_whitespace().collect();
                              let mut link: String = parts[0].to_string();
                              if !link.contains("://") {
                                if link.contains(".gmi") || link.contains(".gemini") {
                                  link = link.replace(".gmi",".html").replace(".gemini",".html");
                                }
                              }
                              if let Ok(Some(kind)) = infer::get_from_path(&path) {
                                if kind.mime_type().starts_with("image/") {
                                  if parts.len() > 1 {
                                    output.push(format!("<IMG SRC={} ALT=\"{}\">",link,parts[1..].join(" ")));
                                  } else {
                                    output.push(format!("<IMG SRC={}>",link));
                                  }
                                } else {
                                  if parts.len() > 1 {
                                    output.push(format!("<A HREF={}>{}</A>",link,parts[1..].join(" ")));
                                  } else {
                                    output.push(format!("<A HREF={}>{}</A>",link,link));
                                  }
                                }
                              }
                            },
                            line if line.starts_with("*") => {
                              let line: &str = line.strip_prefix("*").unwrap_or(&line).trim_start();
                              list.push(format!("<LI>{}</LI>",line))
                            },
                            line if line.starts_with("```") => {
                              output.push(String::from("<PRE>"));
                              raw ^= true;
                            },
                            _   => {
                              if line.is_empty() {
                                output.push(String::from("<BR>"));
                              } else {
                                output.push(format!("<P>{}</P>",line))
                              }
                            },
                          }
                        }
                      },
                      Err(_) => {},
                    }
                  }
                  let head: Vec<String> = vec![
                    String::from("<!DOCTYPE html>"),
                    String::from("<HTML>"),
                    String::from("<HEAD>"),
                    String::from("<META charset=UTF-8>"),
                    header.clone(),
                    String::from("<LINK rel=stylesheet href=/default.css>"),
                    format!("<meta name=description content=\"{}\">",header.clone()),
                    String::from("</HEAD>"),
                    String::from("<BODY>"),
                  ];
                  let tail: Vec<String> = vec![
                    String::from("</BODY>"),
                    String::from("</HTML>"),
                  ];
                  let output: Vec<String> = [head.as_slice(), output.as_slice(), tail.as_slice()].concat();
                  match path.file_stem().and_then(OsStr::to_str) {
                    Some(file) => {
                      let _ = fs::write(&format!("{}/{}.html",self.config.convert_out,file),output.join("\n"));
                    },
                    None => {},
                  }
                },
                Err(_) => {},
              }
            },
            _ => {
              let options = CopyOptions::new();
              let mut paths = Vec::new();
              match path.to_str() {
                Some(path) => {
                  paths.push(path);
                  let _ = copy_items(&paths,&self.config.convert_out,&options);
                },
                None => {},
              }
            },
          }
        },
        Err(_) => {},
      }
    }
    Ok(())
  }
}
