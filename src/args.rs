// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use clap::{Parser, Subcommand, ValueEnum};
use log::LevelFilter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Set the log level
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    pub loglevel: LogLevel,

    /// SQLite Database URL
    #[arg(long, default_value_t = String::from("sqlite:file::memory:?mode=memory&cache=shared"))]
    pub db_url: String,

    /// Directory to save external plugin files. If not set, uses system temp directory.
    #[arg(long)]
    pub ext_plugin_save_dir: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for LevelFilter {
    /// Converts a [`LogLevel`] value into the corresponding `log::LevelFilter` constant.
    ///
    /// # Arguments
    ///
    /// * `level` - The log level specified via the command-line interface.
    ///
    /// # Returns
    ///
    /// Returns the `LevelFilter` variant that matches the provided log level.
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
        }
    }
}

impl From<LogLevel> for tracing::Level {
    /// Converts a [`LogLevel`] into the equivalent `tracing::Level` for the tracing subscriber.
    ///
    /// # Arguments
    ///
    /// * `level` - The user-selected verbosity level.
    ///
    /// # Returns
    ///
    /// Returns the matching `tracing::Level` variant.
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

impl std::fmt::Display for LogLevel {
    /// Formats the [`LogLevel`] as its lowercase string representation for human-readable output.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter receiving the rendered log level.
    ///
    /// # Returns
    ///
    /// Returns `fmt::Result::Ok` when the level string is written successfully, or an error propagated from the formatter.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        };
        write!(f, "{s}")
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the gRPC server
    Start,
}
