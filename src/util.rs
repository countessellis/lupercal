use std::io::{Write,stdout,stdin};
use std::fs;

pub(crate) fn prompt(prompt: String, default: String) -> String {
  print!("{} ",prompt);
  stdout().flush().expect("Oups");
  let mut response = String::new();
  let _ = stdin().read_line(&mut response);
  if response.trim().is_empty() { response = default };
  response.trim_end().to_string()
}

pub(crate) fn build_path(path: &String, prefix: &String) -> String {
  let prefix: String = prefix.strip_suffix('/').unwrap_or(prefix).to_string();
  let mut parts: Vec<&str> = path.split("/").collect();
  if !prefix.is_empty() {
    parts.reverse();
    parts.push(prefix.as_str());
    parts.reverse();
  }
  let directory: String = parts[..parts.len()-1].join("/");
  let file: String = parts[parts.len()-1].to_string();
  if directory.is_empty() {
    file.clone()
  } else {
    match fs::exists(&directory) {
      Ok(true) => directory+"/"+&file,
      Ok(false) => {
        match fs::create_dir_all(&directory) {
          Ok(()) => directory+"/"+&file,
          Err(_) => path.clone(),
        }
      },
      Err(_) => path.clone(),
    }
  }
}
