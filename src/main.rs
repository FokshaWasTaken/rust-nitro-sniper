mod logging;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate serenity;
extern crate tokio;
extern crate dotenv;
extern crate hyper;
extern crate hyper_tls;
extern crate regex;
extern crate fern;
extern crate chrono;
extern crate colored;

use logging::*;
use colored::*;
use std::env;
use dotenv::dotenv;
use hyper_tls::HttpsConnector;
use hyper::{Body, Client, Method, Request, StatusCode};
use hyper::client::HttpConnector;
use regex::Regex;
use serenity::model::channel::Message;
use serenity::prelude::{Context, EventHandler};
use serenity::Client as DiscordClient;
use serenity::async_trait;
use serenity::model::gateway::Ready;

struct Handler {
    client: Client<HttpsConnector<HttpConnector>>,
    main_token: String,
}

impl Handler {
    async fn make_request(&self, gift_token: String, channel_id: String) {
        let request = Request::builder()
            .method(Method::POST)
            .uri(format!(
                "https://discordapp.com/api/v6/entitlements/gift-codes/{}/redeem",
                gift_token
            ))
            .header("Content-Type", "application/json")
            .header("Authorization", self.main_token.clone())
            .body(Body::from(format!("{{\"channel_id\":{}}}", channel_id)))
            .unwrap();

        if let Ok(response) = self.client.request(request).await {
            match response.status() {
                StatusCode::OK => pretty_info!("o(≧▽≦)o", "Yay! Claimed code!"),
                StatusCode::METHOD_NOT_ALLOWED => pretty_error!("(＃＞＜)", "There was an error on Discord's side."),
                StatusCode::NOT_FOUND => pretty_error!("(╥ω╥)", "Code was fake."),
                StatusCode::BAD_REQUEST => pretty_error!("(T_T)", "Code was already redeemed."),
                StatusCode::TOO_MANY_REQUESTS => pretty_warn!("(x_x)", "We were rate-limited..."),
                _ => pretty_error!("┐('～`;)┌", "Received unknown response...")
            }
        } else {
            pretty_warn!("(¬_¬ )", "Requesting failed. Check your connection!");
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, msg: Message) {
        lazy_static! {
            static ref GIFT_PATTERN: Regex = Regex::new("(discord.com/gifts/|discordapp.com/gifts/|discord.gift/)([a-zA-Z0-9]{16})([ ,.]|$)").unwrap();
        }

        if let Some(captures) = GIFT_PATTERN.captures(&msg.content) {
            let gift_token = captures.get(2).unwrap().as_str().to_string();
            pretty_info!("(°ロ°)!", "Found possible gift token: {}! Trying to claim...", gift_token);
            self.make_request(gift_token, msg.channel_id.to_string()).await;
        }
    }

    async fn ready(&self, _ctx: Context, data: Ready) {
        let user = format!("{}#{}", data.user.name, data.user.discriminator);
        pretty_info!("(o･ω･o)", "Connected as {}!", user.as_str().magenta().bold());
        pretty_info!("( ´-ω･)︻┻┳══━一", "Sniping in {} guilds...", data.guilds.len().to_string().as_str().magenta().bold());
    }
}

#[tokio::main]
async fn main() {
    set_up_logger().expect("(o_O) Failed setting up logger. (HOW?)");

    dotenv().ok();
    let main_token = env::var("MAIN_TOKEN")
        .map_err(|_| log_error_and_exit("(￣ω￣;)", "Couldn't find your main token. Please check README!"))
        .unwrap();

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);

    let handler = Handler { client, main_token: main_token.clone() };

    pretty_info!("(ﾉ´ з `)ノ", "Connecting to account.");

    let mut discord_client = DiscordClient::new(&main_token)
        .event_handler(handler)
        .await
        .map_err(|_| log_error_and_exit("(-_-;)・・・", "Couldn't instantiate Discord client."))
        .unwrap();

    discord_client
        .start()
        .await
        .map_err(|_| log_error_and_exit("(＃`Д´)", "Couldn't make a connection to Discord."))
        .unwrap()
}