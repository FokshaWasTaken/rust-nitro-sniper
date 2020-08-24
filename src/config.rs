use crate::{log_error_and_exit, pretty_error};
use colored::*;
use serenity::model::id::GuildId;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Deserialize, Serialize)]
pub struct Config {
    main_token: String,
    snipe_on_main_token: bool,
    sub_tokens: Vec<String>,
    webhook: String,
    guild_blacklist: Vec<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            main_token: "YOUR_TOKEN_HERE".to_string(),
            snipe_on_main_token: true,
            sub_tokens: Vec::new(),
            webhook: "".to_string(),
            guild_blacklist: Vec::new(),
        }
    }
}

impl Config {
    pub fn main_token(&self) -> String {
        self.main_token.clone()
    }

    pub fn get_all_sniping_tokens(&self) -> Vec<String> {
        let mut tokens = self.sub_tokens.clone();
        if self.snipe_on_main_token {
            tokens.insert(0, self.main_token());
        }
        tokens
    }

    pub fn webhook(&self) -> Option<String> {
        if self.webhook != "" {
            Some(self.webhook.clone())
        } else {
            None
        }
    }

    pub fn is_guild_blacklisted(&self, id: Option<GuildId>) -> bool {
        id.map_or_else(|| false, |i| self.guild_blacklist.contains(i.as_u64()))
    }
}

pub enum ConfigReadError {
    NoSuchFile,
    FailedReading,
    MalformedConfig(String),
}

impl ConfigReadError {
    pub fn handle(&self) {
        match self {
            ConfigReadError::NoSuchFile => match create_config() {
                Ok(_) => {
                    log_error_and_exit!(
                            "┐(¯ω¯;)┌",
                            "No previous config file found. Please change your configuration in the rns-config.json file I just created!"
                        );
                }
                Err(_) => {
                    log_error_and_exit!(
                            "┐(¯ω¯;)┌",
                            "No previous config file found. Please create an rns-config.json file with your configuration!"
                        );
                }
            },
            ConfigReadError::MalformedConfig(reason) => {
                pretty_error!(
                    "┐(¯ω¯;)┌",
                    "I couldn't read you config. Did you format it correctly?"
                );
                log_error_and_exit!("->", "...{}.", reason);
            }
            ConfigReadError::FailedReading => {
                log_error_and_exit!("┐(¯ω¯;)┌", "I wasn't able to open your config...");
            }
        }
    }
}

enum ConfigWriteError {
    FailedCreating,
    FailedWriting,
}

pub fn try_read_config() -> Result<Config, ConfigReadError> {
    let mut file = File::open("rns-config.json").map_err(|_| ConfigReadError::NoSuchFile)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|_| ConfigReadError::FailedReading)?;
    serde_json::from_str::<Config>(&contents)
        .map_err(|e| ConfigReadError::MalformedConfig(e.to_string()))
}

fn create_config() -> Result<(), ConfigWriteError> {
    let default_config = Config::default();
    let mut file = File::create("rns-config.json").map_err(|_| ConfigWriteError::FailedCreating)?;
    file.write_all(
        serde_json::to_string_pretty(&default_config)
            .unwrap()
            .as_bytes(),
    )
    .map_err(|_| ConfigWriteError::FailedWriting)?;
    Ok(())
}
