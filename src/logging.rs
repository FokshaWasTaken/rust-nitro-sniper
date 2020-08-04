use log::SetLoggerError;
use fern::colors::{ColoredLevelConfig, Color};
use colored::*;

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

pub fn log_error_and_exit(kaomoji: &str, error: &str) {
    error!("{}{}", kaomoji.bright_white().bold(), error);
    std::process::exit(1);
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