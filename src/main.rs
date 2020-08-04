mod config;
mod handler;
#[macro_use]
mod logging;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate chrono;
extern crate colored;
extern crate fern;
extern crate hyper;
extern crate hyper_tls;
extern crate regex;
extern crate serde;
extern crate serenity;
extern crate tokio;

use crate::logging::log_error_and_exit;
use crate::handler::Handler;
use colored::*;
use hyper::{Body, Client};
use hyper_tls::HttpsConnector;
use serenity::Client as DiscordClient;

#[tokio::main]
async fn main() {
    logging::set_up_logger().expect("(o_O) Failed setting up logger. (HOW?)");

    let config = config::try_read_config()
        .map_err(|e| e.handle())
        .unwrap();

    let main_token = config.main_token;

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);

    let handler = Handler {
        client,
        main_token: main_token.clone(),
    };

    pretty_info!("(o·ω·o)", "Connecting to account.");

    let mut discord_client = DiscordClient::new(&main_token)
        .event_handler(handler)
        .await
        .map_err(|_| log_error_and_exit("(-_-;)°°°", "Couldn't instantiate Discord client."))
        .unwrap();

    discord_client
        .start()
        .await
        .map_err(|_| log_error_and_exit("(＃`Д´)", "Couldn't make a connection to Discord. Is your token correct?"))
        .unwrap();
}
