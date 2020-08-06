mod config;
mod discord;
#[macro_use]
mod logging;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate colored;
extern crate fern;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate serenity;
extern crate tokio;

use colored::*;
use hyper::{Body, Client};
use hyper_tls::HttpsConnector;
use serenity::Client as DiscordClient;

#[tokio::main]
async fn main() {
    logging::set_up_logger().expect("(o_O) Failed setting up logger. (HOW?)");

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);

    let config = config::try_read_config().map_err(|e| e.handle()).unwrap();
    let main_token = config.main_token();
    let main_profile = discord::get_profile_for_token(&main_token, &client)
        .await
        .map_err(|e| e.handle())
        .unwrap();

    pretty_info!(
        "o(»ω«)o",
        "If we're lucky you'll get Nitro on {}!",
        main_profile
    );

    let mut sniping_tokens = config.get_all_sniping_tokens();
    sniping_tokens.sort();
    sniping_tokens.dedup();

    if sniping_tokens.is_empty() {
        log_error_and_exit!(
            "┐(¯ω¯;)┌",
            "...but I need at least one token to snipe with..."
        );
    }

    pretty_info!(
        "(o·ω·o)",
        "I'll be sniping on {} account(s)! Please wait until I connect to them...",
        sniping_tokens.len()
    );

    let handler = discord::Handler {
        client,
        main_token: main_token.clone(),
    };

    let mut tasks = Vec::new();

    for (index, token) in sniping_tokens.iter().enumerate() {
        let discord_client_result = DiscordClient::new(token)
            .event_handler(handler.clone())
            .await;

        if let Ok(mut discord_client) = discord_client_result {
            tasks.push(tokio::spawn(async move {
                let connection_result = discord_client.start().await;
                if connection_result.is_err() {
                    pretty_error!(
                        "(＃`Д´)",
                        "Couldn't make a connection to Discord on token #{}. Is your token correct?",
                        index
                    );
                }
            }));
        } else {
            pretty_error!(
                "(-_-;)°°°",
                "Couldn't instantiate a Discord client for token #{}.",
                index,
            );
        }
    }

    futures::future::join_all(tasks).await;
    log_error_and_exit!("(x_x)", "Lost all connections.");
}
