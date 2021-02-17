mod cmd;

use std::fs;

use fern::Dispatch;
use log::LevelFilter;
use serde_json::{to_string_pretty, Value};
use tauri::plugin::Plugin;

use crate::config::{read_config, TomlConfig};
use cmd::Cmd;
pub use cmd::LogLevel;

pub struct Logger;

impl Logger {
  pub fn new() -> Self {
    Logger::setup_logger().unwrap();
    Logger
  }

  fn setup_logger() -> Result<(), fern::InitError> {
    let log_file = format!(
      // TODO cross platform solution
      "/tmp/cabr2_{}.log",
      chrono::Local::now().format("%F_%H.%M.%S")
    );

    let config = read_config()
      .unwrap_or_else(|e| {
        eprintln!("loading config failed: {}", e);
        eprintln!("continuing with default config");
        TomlConfig::default()
      })
      .logging;

    Dispatch::new()
      .format(|out, message, record| {
        out.finish(format_args!(
          "{}[{}][{}] {}",
          chrono::Local::now().format("[%+]"),
          record.target(),
          record.level(),
          message
        ))
      })
      .level(convert_level(config.all))
      .level_for("cabr2", convert_level(config.cabr2))
      .level_for("ureq", convert_level(config.ureq))
      .level_for("rustls", convert_level(config.rustls))
      .chain(std::io::stdout())
      .chain(fs::OpenOptions::new().create(true).write(true).open(&log_file)?)
      .apply()?;
    log::info!("log file: {}", log_file);
    Ok(())
  }

  fn handle(&self, level: LogLevel, messages: Vec<Value>) -> Result<(), String> {
    let mut formatted_messages = Vec::with_capacity(messages.len());
    for message in messages {
      match message {
        Value::String(s) => formatted_messages.push(s),
        _ => match to_string_pretty(&message) {
          Ok(formatted) => formatted_messages.push(formatted),
          Err(e) => return Err(e.to_string()),
        },
      }
    }

    let print_message = formatted_messages.join(" ");
    match level {
      LogLevel::TRACE => log::trace!("{}", print_message),
      LogLevel::DEBUG => log::debug!("{}", print_message),
      LogLevel::INFO => log::info!("{}", print_message),
      LogLevel::WARNING => log::warn!("{}", print_message),
      LogLevel::ERROR => log::error!("{}", print_message),
    }

    Ok(())
  }
}

/// Converts an `Option<LogLevel>` into `LevelFilter`.
/// If the Option is `None` the filter is set to `Error`.
fn convert_level(level: Option<LogLevel>) -> LevelFilter {
  match level {
    Some(level) => level.into(),
    None => LevelFilter::Error,
  }
}

impl Plugin for Logger {
  fn extend_api(&self, _: &mut tauri::Webview, payload: &str) -> Result<bool, String> {
    match serde_json::from_str(payload) {
      Err(e) => Err(e.to_string()),
      Ok(command) => {
        match command {
          Cmd::Log { level, messages } => self.handle(level, messages)?,
        }
        Ok(true)
      }
    }
  }
}

impl std::convert::From<LogLevel> for LevelFilter {
  fn from(level: LogLevel) -> Self {
    match level {
      LogLevel::TRACE => LevelFilter::Trace,
      LogLevel::DEBUG => LevelFilter::Debug,
      LogLevel::INFO => LevelFilter::Info,
      LogLevel::WARNING => LevelFilter::Warn,
      LogLevel::ERROR => LevelFilter::Error,
    }
  }
}
