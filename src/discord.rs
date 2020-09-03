use crate::cache::LocationCache;
use crate::config::Config;
use crate::logging::LogBlock;
use crate::matcher::get_gift_code;
use crate::util::user_to_tag;
use crate::webhook::Webhook;
use crate::{log_error_and_exit, pretty_error, pretty_info, pretty_success, pretty_warn};
use colored::*;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;
use once_cell::sync::OnceCell;
use serenity::async_trait;
use serenity::http::{CacheHttp, GuildPagination, Http};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::model::user::CurrentUser;
use serenity::prelude::{Context, EventHandler};
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

type HttpsClient = Client<HttpsConnector<HttpConnector>>;

pub struct HandlerInfo {
    client: HttpsClient,
    config: Config,
    seen_codes: Mutex<Vec<String>>,
    token_amount: usize,
    connected: AtomicUsize,
    total_guilds: AtomicUsize,
}

impl HandlerInfo {
    pub fn new(client: HttpsClient, config: Config, token_amount: usize) -> Self {
        HandlerInfo {
            client,
            config,
            seen_codes: Mutex::new(Vec::new()),
            token_amount,
            connected: AtomicUsize::new(0),
            total_guilds: AtomicUsize::new(0),
        }
    }
}

pub struct Handler {
    initialized: AtomicBool,
    profile: OnceCell<Profile>,
    location_cache: LocationCache,
    info: Arc<HandlerInfo>,
}

impl Handler {
    pub fn new(info: Arc<HandlerInfo>) -> Self {
        Handler {
            initialized: AtomicBool::new(false),
            profile: OnceCell::new(),
            location_cache: LocationCache::new(),
            info,
        }
    }

    async fn make_request(&self, gift_code: String, message: &Message, log: &mut LogBlock<'_>) {
        let request = Request::builder()
            .method(Method::POST)
            .uri(format!(
                "https://discordapp.com/api/v8/entitlements/gift-codes/{}/redeem",
                gift_code
            ))
            .header("Authorization", &self.info.config.main_token())
            .header("Content-Length", 0)
            .body(Body::empty())
            .unwrap();

        if let Ok(response) = self.info.client.request(request).await {
            match response.status() {
                StatusCode::OK => self.on_success(message, log).await,
                StatusCode::METHOD_NOT_ALLOWED => {
                    pretty_error!(log: log, "(x_x)", "There was an error on Discord's side.");
                }
                StatusCode::NOT_FOUND => {
                    pretty_warn!(log: log, "(╥ω╥)", "Code was fake or expired.");
                }
                StatusCode::BAD_REQUEST => {
                    pretty_error!(log: log, "(╥ω╥)", "Code was already redeemed.");
                }
                StatusCode::TOO_MANY_REQUESTS => {
                    pretty_warn!(log: log, "(x_x)", "We were rate-limited...");
                }
                unknown => {
                    pretty_error!(
                        log: log,
                        "┐(¯ω¯;)┌",
                        "Received unknown response... ({}{})",
                        unknown.as_str(),
                        unknown
                            .canonical_reason()
                            .map_or_else(|| "".to_string(), |r| format!(" {}", r))
                    );
                    if let Ok(Ok(body)) = hyper::body::to_bytes(response.into_body())
                        .await
                        .map(|b| String::from_utf8(b.to_vec()))
                    {
                        pretty_error!(log: log, "->", "...with this body: {}", body);
                    } else {
                        pretty_error!(
                            log: log,
                            "->",
                            "...and couldn't parse the body of the response."
                        );
                    }
                }
            }
        } else {
            pretty_warn!(
                log: log,
                "┐(¯ω¯;)┌",
                "Requesting failed. Check your connection!"
            );
        }
    }

    async fn on_success(&self, message: &Message, log: &mut LogBlock<'_>) {
        pretty_success!(log: log, "o(»ω«)o", "Yay! Claimed code!");
        if let Some(webhook_url) = self.info.config.webhook() {
            pretty_success!(log: log, "(o·ω·o)", "Sending webhook message!");
            let webhook = Webhook::new(webhook_url);
            if webhook
                .send(message, &self.info.client, self.profile.get().unwrap())
                .await
                .is_err()
            {
                pretty_error!(log: log, "┐(¯ω¯;)┌", "Failed sending webhook message");
            }
        }
    }

    fn initialize(&self, profile: Profile, guild_amount: usize) {
        pretty_info!(
            "(o·ω·o)",
            "Connected as {}! Now sniping in {} guilds...",
            profile.to_string().as_str().magenta().bold(),
            guild_amount.to_string().as_str().magenta().bold()
        );
        self.profile.set(profile).unwrap();
        self.info
            .total_guilds
            .fetch_add(guild_amount, Ordering::Relaxed);

        if self.info.connected.fetch_add(1, Ordering::Relaxed) + 1 == self.info.token_amount
            && self.info.token_amount > 1
        {
            pretty_info!(
                "( ´-ω·)±┻┳══━─",
                "Connected to all {} accounts! Sniping in {} guilds in total!",
                self.info.token_amount,
                self.info.total_guilds.load(Ordering::Relaxed)
            );
        }
    }

    async fn initialize_from_raw(&self, http: &Http) {
        self.initialized.store(true, Ordering::Relaxed);
        let profile = Profile::from(http.get_current_user().await.unwrap());
        let guild_amount = http
            .get_guilds(&GuildPagination::After(GuildId(0)), 100)
            .await
            .unwrap()
            .len();
        self.initialize(profile, guild_amount);
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !self.initialized.load(Ordering::Relaxed) {
            self.initialize_from_raw(ctx.http()).await;
        } else if self.profile.get().is_none() {
            return;
        }

        if !self.info.config.is_guild_blacklisted(msg.guild_id) {
            if let Some(gift_code) = get_gift_code(&msg) {
                let mut seen_codes = self.info.seen_codes.lock().await;
                if !seen_codes.contains(&gift_code) {
                    seen_codes.push(gift_code.clone());

                    let mut log = LogBlock::new(self.profile.get().unwrap());
                    pretty_info!(log: log, "(°■°)!", "Claiming code: {}!", gift_code);

                    self.make_request(gift_code, &msg, &mut log).await;
                    log.freeze_time();

                    let location = self
                        .location_cache
                        .get_and_cache_location(msg.channel_id, msg.guild_id, ctx.http())
                        .await;
                    log.send(location, user_to_tag(&msg.author));
                }
            }
        }
    }

    async fn ready(&self, _ctx: Context, data: Ready) {
        if self.initialized.load(Ordering::Relaxed) {
            return;
        }
        self.initialized.store(true, Ordering::Relaxed);
        self.initialize(Profile::from(data.user), data.guilds.len());
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Profile {
    username: String,
    discriminator: String,
    avatar: Option<String>,
    id: String,
}

impl Profile {
    fn get_avatar(&self) -> Option<String> {
        self.avatar.clone()
    }

    pub fn face(&self) -> String {
        self.get_avatar().map_or_else(
            || "https://discordapp.com/assets/6debd47ed13483642cf09e832ed0bc1b.png".to_string(),
            |a| format!("https://cdn.discordapp.com/avatars/{}/{}.webp", self.id, a),
        )
    }
}

impl From<CurrentUser> for Profile {
    fn from(user: CurrentUser) -> Self {
        Profile {
            username: user.name,
            discriminator: format!("{:0>4}", user.discriminator),
            avatar: user.avatar,
            id: user.id.to_string(),
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}#{}", self.username, self.discriminator)
    }
}

pub enum ProfileError {
    Unauthorized,
    RateLimited,
    ConnectionError,
    Other,
}

impl ProfileError {
    pub fn handle(&self) {
        match self {
            ProfileError::Unauthorized => {
                log_error_and_exit!(
                    "┐(¯ω¯;)┌",
                    "I couldn't verify your main token. Is it correct?"
                );
            }
            ProfileError::RateLimited => {
                log_error_and_exit!("(x_x)", "Your're rate-limited. Try again later...");
            }
            ProfileError::ConnectionError => {
                log_error_and_exit!("┐(¯ω¯;)┌", "Requesting failed. Check your connection!");
            }
            ProfileError::Other => {
                log_error_and_exit!("┐(¯ω¯;)┌", "Received unknown response for Discord...");
            }
        }
    }
}

pub async fn get_profile_for_token(
    token: &str,
    client: &HttpsClient,
) -> Result<Profile, ProfileError> {
    let request = Request::builder()
        .method(Method::GET)
        .uri("https://discordapp.com/api/v6/users/@me")
        .header("Authorization", token)
        .body(Body::empty())
        .unwrap();

    let response_result = client.request(request).await;

    if let Ok(response) = response_result {
        match response.status() {
            StatusCode::OK => {
                let streamed_bytes = hyper::body::to_bytes(response.into_body()).await;
                if let Ok(bytes) = streamed_bytes {
                    let body = String::from_utf8(bytes.to_vec()).expect("Received bad stream.");
                    let profile = serde_json::from_str(&body).expect("Malformed response.");
                    Ok(profile)
                } else {
                    Err(ProfileError::Other)
                }
            }
            StatusCode::UNAUTHORIZED => Err(ProfileError::Unauthorized),
            StatusCode::TOO_MANY_REQUESTS => Err(ProfileError::RateLimited),
            _ => Err(ProfileError::Other),
        }
    } else {
        Err(ProfileError::ConnectionError)
    }
}
