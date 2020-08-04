use crate::logging::log_error_and_exit;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub main_token: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            main_token: "YOUR_TOKEN_HERE".to_string(),
        }
    }
}

pub enum ConfigReadError {
    NoSuchFile,
    FailedReading,
    MalformedConfig,
}

impl ConfigReadError {
    pub fn handle(&self) {
        match self {
            ConfigReadError::NoSuchFile => match create_config() {
                Ok(_) => {
                    log_error_and_exit(
                            "┐(¯ω¯;)┌",
                            "No previous config file found. Please change your configuration in the rna-config.json file I just created!"
                        );
                }
                Err(_) => {
                    log_error_and_exit(
                            "┐(¯ω¯;)┌",
                            "No previous config file found. Please create an rna-config.json file with your configuration!"
                        );
                }
            },
            ConfigReadError::MalformedConfig => {
                log_error_and_exit(
                    "┐(¯ω¯;)┌",
                    "I couldn't read you config. Did you format it correctly?",
                );
            }
            ConfigReadError::FailedReading => {
                log_error_and_exit("┐(¯ω¯;)┌", "I wasn't able to open your config...");
            }
        }
    }
}

enum ConfigWriteError {
    FailedCreating,
    FailedWriting,
}

pub fn try_read_config() -> Result<Config, ConfigReadError> {
    let mut file = File::open("rna-config.json").map_err(|_| ConfigReadError::NoSuchFile)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|_| ConfigReadError::FailedReading)?;
    serde_json::from_str::<Config>(&contents).map_err(|_| ConfigReadError::MalformedConfig)
}

fn create_config() -> Result<(), ConfigWriteError> {
    let default_config = Config::default();
    let mut file = File::create("rna-config.json").map_err(|_| ConfigWriteError::FailedCreating)?;
    file.write_all(serde_json::to_string_pretty(&default_config).unwrap().as_bytes())
        .map_err(|_| ConfigWriteError::FailedWriting)?;
    Ok(())
}
