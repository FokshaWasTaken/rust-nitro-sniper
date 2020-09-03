use serde::export::Formatter;
use serenity::http::Http;
use serenity::model::channel::Channel;
use serenity::model::id::{ChannelId, GuildId};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Location {
    pub guild_name: Option<Arc<String>>,
    pub channel_name: String,
}

impl Default for Location {
    fn default() -> Self {
        Location {
            guild_name: None,
            channel_name: "?".to_string(),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(guild_name) = &self.guild_name {
            write!(f, "{} > {}", guild_name, self.channel_name)
        } else {
            write!(f, "{}", self.channel_name)
        }
    }
}

impl Location {
    pub fn new(guild_name: Option<Arc<String>>, channel: Channel) -> Self {
        let channel_name = match channel {
            Channel::Guild(guild_channel) => guild_channel.name,
            Channel::Private(private_channel) => private_channel.name(),
            Channel::Group(group) => group.name().to_string(),
            _ => unreachable!(),
        };

        Location {
            guild_name,
            channel_name,
        }
    }
}

pub struct LocationCache {
    channel_map: Mutex<HashMap<ChannelId, Location>>,
    guild_map: Mutex<HashMap<GuildId, Arc<String>>>,
}

impl LocationCache {
    pub fn new() -> Self {
        LocationCache {
            channel_map: Mutex::new(HashMap::new()),
            guild_map: Mutex::new(HashMap::new()),
        }
    }

    async fn make_location_request(
        &self,
        channel_id: ChannelId,
        guild_id: Option<GuildId>,
        http: &Http,
    ) -> Result<Location, ()> {
        let guild_result = if let Some(id) = guild_id {
            Some(self.get_and_cache_guild(id, http).await)
        } else {
            None
        };

        let channel_result = http.get_channel(channel_id.0).await;

        let location = match (guild_result, channel_result) {
            (Some(Ok(guild_name)), Ok(channel)) => Location::new(Some(guild_name), channel),
            (None, Ok(channel)) => Location::new(None, channel),
            _ => return Err(()),
        };

        Ok(location)
    }

    pub async fn get_and_cache_location(
        &self,
        channel_id: ChannelId,
        guild_id: Option<GuildId>,
        http: &Http,
    ) -> Result<Location, ()> {
        let mut channel_map = self.channel_map.lock().await;

        match channel_map.entry(channel_id) {
            Entry::Vacant(entry) => {
                if let Ok(response) = self.make_location_request(channel_id, guild_id, http).await {
                    Ok(entry.insert(response).clone())
                } else {
                    Err(())
                }
            }
            Entry::Occupied(entry) => Ok(entry.get().clone()),
        }
    }

    pub async fn get_and_cache_guild(
        &self,
        guild_id: GuildId,
        http: &Http,
    ) -> Result<Arc<String>, ()> {
        let mut guild_map = self.guild_map.lock().await;

        match guild_map.entry(guild_id) {
            Entry::Vacant(entry) => {
                if let Ok(response) = http.get_guild(guild_id.0).await {
                    Ok(entry.insert(Arc::new(response.name)).clone())
                } else {
                    Err(())
                }
            }
            Entry::Occupied(entry) => Ok(entry.get().clone()),
        }
    }
}
