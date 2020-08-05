use colored::*;
use fern::colors::{Color, ColoredLevelConfig};
use log::SetLoggerError;
use std::io::{stdin, stdout, Read, Write};

pub fn set_up_logger() -> Result<(), SetLoggerError> {
    let colors = ColoredLevelConfig::new()
        .error(Color::BrightMagenta)
        .info(Color::Cyan)
        .debug(Color::BrightBlack)
        .warn(Color::Yellow)
        .trace(Color::Blue);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "({})› ({}) {}",
                chrono::Local::now().format("%H:%M:%S"),
                colors.color(record.level()),
                message
            ))
        })
        .level_for("serenity", log::LevelFilter::Off)
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;

    println!(
        "             {} {} {}",
        "=^.^=".blue().bold(),
        "Welcome to".magenta(),
        "RNS!".bright_magenta().bold()
    );

    Ok(())
}

pub fn pause_exit() {
    let mut stdout = stdout();
    stdout.write(b"Press the enter key to exit...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
    std::process::exit(1);
}

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
    )
}

#[macro_export]
macro_rules! pretty_warn {
    ($e:tt, $($arg:tt)+) => (
        warn!("{} {}", $e.bright_white().bold(), format!($($arg)+));
    )
}

#[macro_export]
macro_rules! pretty_error {
    ($e:tt, $($arg:tt)+) => (
        error!("{} {}", $e.bright_white().bold(), format!($($arg)+));
    )
}

#[macro_export]
macro_rules! pretty_success {
    ($e:tt, $($arg:tt)+) => (
        error!("{} {}", $e.bright_green().bold(), format!($($arg.green())+));
    )
}
