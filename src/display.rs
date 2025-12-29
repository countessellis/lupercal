use std::time::Duration;
use color_eyre::{eyre::Context, Result};
use ratatui::{
  crossterm::event::{self, Event, KeyCode},
  widgets::Paragraph,
  DefaultTerminal, Frame,
};
use ratatui::text::Line;
use ratatui::style::Style;
use ratatui::style::Modifier;
use crate::display::event::KeyEventKind;
use url::Url;

use crate::client::*;
use crate::response::*;
use crate::util;

///////////// Display

pub(crate) fn display(client: &Client, source: &Url, response: &Response) -> Result<()> {
  let payload: Option<(String,String)> = response.text();
  match payload {
    Some((subtype,text)) => {
      color_eyre::install()?;
      let mut terminal = ratatui::init();
      let lines: Vec<&str> = text.lines().collect();
      let (lines,links) = format(lines);
      let paragraph: Paragraph = Paragraph::new(lines);
      loop {
        terminal.draw(|frame| {
          frame.render_widget(&paragraph,frame.area());
        });
        if event::poll(Duration::from_millis(250)).context("event poll failed")? {
          if let Event::Key(key) = event::read().context("event read failed")? {
            if key.kind == KeyEventKind::Press {
              if let KeyCode::Char(c) = key.code {
                if let Some(digit) = c.to_digit(10) {
                  let link: String = links[digit as usize - 1].clone();
                  let request: String = util::build_abs_url(&source,&link);
                  log::debug!("Link {} chosen, link is: {}",digit,request);
                  ratatui::restore();
                  client.request(&request);
                } else {
                  match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Esc       => break,
                    _                  => {},
                  }
                }
              }
            }
          }
        }
      }
      ratatui::restore();
    },
    None => {},
  }
  Ok(())
}

fn format(lines: Vec<&str>) -> (Vec<Line>,Vec<String>) {
  let mut formatted: Vec<Line> = Vec::new();
  let mut links: Vec<String> = Vec::new();
  for line in lines {
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
  }
  (formatted,links)
}

fn should_quit() -> Result<bool> {
  if event::poll(Duration::from_millis(250)).context("event poll failed")? {
    if let Event::Key(key) = event::read().context("event read failed")? {
      return Ok(KeyCode::Char('q') == key.code);
    }
  }
  Ok(false)
}
