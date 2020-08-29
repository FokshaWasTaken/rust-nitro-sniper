use crate::cache::Location;
use crate::discord::Profile;
use colored::*;
use log::{Level, SetLoggerError};
use std::io::{stdin, stdout, Read, Write};
use std::time::Instant;

#[macro_export]
macro_rules! log_error_and_exit {
    ($e:tt, $($arg:tt)+) => (
        error!("{} {}", $e.bright_white().bold(), format!($($arg)+));
        crate::logging::pause_exit();
    )
}

#[macro_export]
macro_rules! pretty_info {
    ($e:tt, $($arg:tt)+) => (
        info!("{} {}", $e.bright_white().bold(), format!($($arg)+));
    );
    (log: $log:tt, $e:tt, $($arg:tt)+) => (
        let text = format!($($arg)+);
        $log.add_message(log::Level::Info, $e, text, false);
    )
}

#[macro_export]
macro_rules! pretty_warn {
    ($e:tt, $($arg:tt)+) => (
        warn!("{} {}", $e.bright_white().bold(), format!($($arg)+));
    );
    (log: $log:tt, $e:tt, $($arg:tt)+) => (
        let text = format!($($arg)+);
        $log.add_message(log::Level::Warn, $e, text, false);
    )
}

#[macro_export]
macro_rules! pretty_error {
    ($e:tt, $($arg:tt)+) => (
        error!("{} {}", $e.bright_white().bold(), format!($($arg)+));
    );
    (log: $log:tt, $e:tt, $($arg:tt)+) => (
        let text = format!($($arg)+);
        $log.add_message(log::Level::Error, $e, text, false);
    )
}

#[macro_export]
macro_rules! pretty_success {
    ($e:tt, $($arg:tt)+) => (
        info!("{} {}", $e.bright_green().bold(), format!("{}", $($arg.green())+));
    );
    (log: $log:tt, $e:tt, $($arg:tt)+) => (
        let text = format!($($arg)+);
        $log.add_message(log::Level::Info, $e, text, true);
    )
}

pub struct LogBlock<'a> {
    messages: Vec<LogMessage<'a>>,
    start: Instant,
    profile: &'a Profile,
    elapsed: Option<u128>,
}

impl<'a> LogBlock<'a> {
    pub fn new(profile: &'a Profile) -> Self {
        LogBlock {
            messages: Vec::new(),
            start: Instant::now(),
            profile,
            elapsed: None,
        }
    }

    pub fn add_message(&mut self, level: Level, kaomoji: &'a str, text: String, is_success: bool) {
        let message = LogMessage {
            kaomoji,
            text,
            level,
            is_success,
        };
        self.messages.push(message);
    }

    pub fn freeze_time(&mut self) {
        self.elapsed = Some(self.start.elapsed().as_millis());
    }

    pub fn send(&mut self, location_cache: Result<Location, ()>, sender: String) {
        if self.elapsed.is_none() {
            self.freeze_time();
        }

        let location = if let Ok(location) = location_cache {
            location
        } else {
            pretty_error!(log: self, "(x_x)", "Failed requesting location for event.");
            Location::default()
        };

        println!(
            "\n{} › ({}) [{} > {}]",
            chrono::Local::now().format("%H:%M:%S"),
            self.profile,
            location,
            sender
        );

        for message in &self.messages {
            message.send();
        }

        println!("Finished in: {}ms", self.elapsed.unwrap());
    }
}

pub struct LogMessage<'a> {
    kaomoji: &'a str,
    text: String,
    level: Level,
    is_success: bool,
}

impl LogMessage<'_> {
    pub fn send(&self) {
        let (text, kaomoji) = if self.is_success {
            (
                self.text.as_str().bright_green(),
                self.kaomoji.bold().bright_green(),
            )
        } else {
            (
                self.text.as_str().normal(),
                self.kaomoji.bold().bright_white(),
            )
        };

        println!(" ({}) {} {}", map_level(self.level), kaomoji, text);
    }
}

fn map_level(level: Level) -> ColoredString {
    match level {
        Level::Info => "+".cyan(),
        Level::Trace => "-".blue(),
        Level::Error => "!".bright_magenta(),
        Level::Debug => "*".bright_black(),
        Level::Warn => "?".yellow(),
    }
}

pub fn set_up_logger() -> Result<(), SetLoggerError> {
    #[cfg(windows)]
    {
        let _ = colored::control::set_virtual_terminal(true);
    }

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}› ({}) {}",
                chrono::Local::now().format("%H:%M:%S"),
                map_level(record.level()),
                message
            ))
        })
        .level_for("serenity", log::LevelFilter::Off)
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;

    println!(
        "{} {} {}\n",
        "=^.^=".blue().bold(),
        "Welcome to".magenta(),
        "RNS!".bright_magenta().bold()
    );

    Ok(())
}

pub fn pause_exit() {
    let mut stdout = stdout();
    stdout.write_all(b"Press the enter key to exit...").unwrap();
    stdout.flush().unwrap();
    stdin().read_exact(&mut [0]).unwrap();
    std::process::exit(1);
}
