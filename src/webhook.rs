use crate::discord::Profile;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;
use serde_json::Value;
use serenity::model::channel::{Embed, Message};

type HttpsClient = Client<HttpsConnector<HttpConnector>>;

pub struct Webhook {
    pub url: String,
}

impl Webhook {
    pub fn new(url: String) -> Self {
        Webhook { url }
    }

    pub async fn send(&self, message: Message, client: &HttpsClient, finder: Profile) -> Result<(), ()> {
        let payload = WebhookPayload::new(message, finder);
        let request = Request::builder()
            .method(Method::POST)
            .uri(&self.url)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        let result = client.request(request).await;
        if let Ok(response) = result {
            if let StatusCode::NO_CONTENT = response.status() {
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

#[derive(Serialize)]
struct WebhookPayload {
    username: String,
    avatar_url: String,
    embeds: Vec<Value>,
}

impl WebhookPayload {
    fn new(message: Message, finder: Profile) -> Self {
        let embed = Embed::fake(|create| {
            create
                .author(|a| a.icon_url(finder.face()).name(finder.to_string()))
                .title("Yay! Claimed a Nitro!")
                .description("o(»ω«)o Congratulations! I'm so proud of you!\nMaybe star me on [GitHub](https://github.com/Melonai/rust-nitro-sniper)? (ﾉ´ヮ´)ﾉ*:･ﾟ✧")
                .field(
                    "Nitro sent by:",
                    format!("{}#{}", message.author.name, message.author.discriminator),
                    false,
                )
                .field(
                    "Message:",
                    format!(
                        "[Posted here!](https://discordapp.com/channels/{}/{}/{})",
                        message
                            .guild_id
                            .map_or_else(|| "@me".to_string(), |g| g.to_string()),
                        message.channel_id.to_string(),
                        message.id.to_string()
                    ),
                    false,
                )
                .footer(|f| {
                    f.icon_url(WebhookPayload::get_rns_avatar())
                        .text(format!("RNS {}", env!("CARGO_PKG_VERSION")))
                })
                .timestamp(chrono::Local::now().to_rfc3339())
                .colour(5973197)
        });
        let embeds = vec![embed];
        WebhookPayload {
            embeds,
            ..Default::default()
        }
    }

    fn get_rns_avatar() -> String {
        "https://i.imgur.com/CxZOLY4.png".to_string()
    }
}

impl Default for WebhookPayload {
    fn default() -> Self {
        WebhookPayload {
            username: "RNS".to_string(),
            avatar_url: WebhookPayload::get_rns_avatar(),
            embeds: Vec::new(),
        }
    }
}
