use std::{ffi::OsStr,fs,fs::File,io::{BufRead,BufReader,ErrorKind},path::Path};
use walkdir::WalkDir;

use crate::config::*;

use crate::util;

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

  pub(crate) fn run(&self) -> Result<(),String> {
    let input = Path::new(&self.config.convert_in);
    let output = Path::new(&self.config.convert_out);
    for entry in WalkDir::new(&self.config.convert_in).into_iter().filter_map(|entry| entry.ok()) {
      let path = entry.path();
      match path.extension().and_then(OsStr::to_str) {
        Some("gemini") | Some("gmi") => {
          match File::open(&path) {
            Ok(file) => {
              let reader = BufReader::new(file);
              match path.file_stem().and_then(OsStr::to_str) {
                Some(file) => {
                  log::info!("Converting {} to {}/{}.html.",path.display(),self.config.convert_out,file);
                  let output: Vec<String> = reader.lines().filter_map(Result::ok).collect();
                  let _ = fs::write(&format!("{}/{}.html",self.config.convert_out,file),Self::convert(&output));
                },
                None => {},
              }
            },
            Err(_) => {},
          }
        },
        _ => {
          match path.strip_prefix(&self.config.convert_in) {
            Ok(rel_path) => {
              let target = output.join(rel_path);
              let target = target.as_path();
              if path == input { continue };
              if path.is_dir() {
                match fs::create_dir_all(&target) {
                  Ok(_)    => log::info!("Created directory {}.",target.display()),
                  Err(err) if err.kind() == ErrorKind::AlreadyExists => log::info!("Directory {} already exists, skipping.",target.display()),
                  Err(err) => log::error!("Failed to create directory {}: {}",target.display(),err),
                }
              } else {
                if target.parent().is_some() {
                  match fs::create_dir_all(&target) {
                    Ok(_)    => log::info!("Created directory {}.",target.display()),
                    Err(err) if err.kind() == ErrorKind::AlreadyExists => log::info!("Directory {} already exists, skipping.",target.display()),
                    Err(err) => log::error!("Failed to create directory {}: {}",target.display(),err),
                  }
                }
                match fs::copy(path, &target) {
                  Ok(_)    => log::info!("Copied {} to {}.",path.display(),target.display()),
                  Err(err) => log::error!("Failed to copy {} to {}: {}",path.display(),target.display(),err),
                }
              }
            },
            Err(err) => log::error!("Failed to get relative path for {}: {}",path.display(),err),
          }
        }
      }
    }
    Ok(())
  }

  pub(crate) fn convert(lines: &Vec<String>) -> String {
    let mut output: Vec<String> = Vec::new();
    let mut raw: bool = false;
    let mut list: Vec<String> = Vec::new();
    let mut header: String = String::new();
    for line in lines.iter() {
      if raw {
        if line.starts_with("```") {
          output.push(String::from("</PRE>"));
          raw ^= true;
        } else {
          output.push(line.to_string());
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
            match util::is_image(&link) {
              Some(_) => {
                if parts.len() > 1 {
                  output.push(format!("<P><IMG SRC={} ALT=\"{}\"></P>",link,parts[1..].join(" ")));
                } else {
                  output.push(format!("<P><IMG SRC={}></P>",link));
                }
              },
              None => {
                if parts.len() > 1 {
                  output.push(format!("<P><A HREF={}>{}</A></P>",link,parts[1..].join(" ")));
                } else {
                  output.push(format!("<P><A HREF={}>{}</A></P>",link,link));
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
    output.join("\n")
  }
}
