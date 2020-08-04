#[macro_use]
extern crate lazy_static;
extern crate serenity;
extern crate tokio;
extern crate dotenv;
extern crate hyper;
extern crate hyper_tls;
extern crate regex;


use std::env;
use dotenv::dotenv;
use hyper_tls::HttpsConnector;
use hyper::{Body, Client, Method, Request};
use hyper::client::HttpConnector;
use hyper::body::to_bytes;
use regex::Regex;
use serenity::model::channel::Message;
use serenity::prelude::{Context, EventHandler};
use serenity::Client as DiscordClient;
use serenity::async_trait;


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
            let bytes = to_bytes(response.into_body()).await.unwrap();
            println!("{}", String::from_utf8(bytes.to_vec()).unwrap());
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
            self.make_request(gift_token, msg.channel_id.to_string()).await;
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let main_token = env::var("MAIN_TOKEN").expect("Expected main token in '.env'.");

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);

    let handler = Handler { client, main_token: main_token.clone() };

    let mut discord_client = DiscordClient::new(&main_token)
        .event_handler(handler)
        .await
        .expect("Failed making a Discord client.");

    discord_client
        .start()
        .await
        .expect("Failed starting the Discord client.");
}
