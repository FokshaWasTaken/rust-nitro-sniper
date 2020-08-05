use crate::{pretty_error, pretty_info, pretty_warn};
use colored::*;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;
use regex::Regex;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};

#[derive(Clone)]
pub struct Handler {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub main_token: String,
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
                StatusCode::OK => pretty_info!("o(»ω«)o", "Yay! Claimed code!"),
                StatusCode::METHOD_NOT_ALLOWED => {
                    pretty_error!("(x_x)", "There was an error on Discord's side.")
                }
                StatusCode::NOT_FOUND => pretty_error!("(╥ω╥)", "Code was fake."),
                StatusCode::BAD_REQUEST => pretty_error!("(╥ω╥)", "Code was already redeemed."),
                StatusCode::TOO_MANY_REQUESTS => pretty_warn!("(x_x)", "We were rate-limited..."),
                _ => pretty_error!("┐(¯ω¯;)┌", "Received unknown response..."),
            }
        } else {
            pretty_warn!("┐(¯ω¯;)┌", "Requesting failed. Check your connection!");
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
            pretty_info!(
                "(°■°)!",
                "Found possible gift token: {}! Trying to claim...",
                gift_token
            );
            self.make_request(gift_token, msg.channel_id.to_string())
                .await;
        }
    }

    async fn ready(&self, _ctx: Context, data: Ready) {
        let user = format!("{}#{}", data.user.name, data.user.discriminator);
        pretty_info!(
            "(o·ω·o)",
            "Connected as {}!",
            user.as_str().magenta().bold()
        );
        pretty_info!(
            "( ´-ω·)±┻┳══━─",
            "...which is now sniping in {} guilds...",
            data.guilds.len().to_string().as_str().magenta().bold()
        );
    }
}
