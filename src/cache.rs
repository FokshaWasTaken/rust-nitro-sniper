use colored::*;
use serenity::http::Http;
use serenity::model::channel::Channel;
use serenity::model::id::{ChannelId, GuildId};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Location {
    pub guild_name: Option<String>,
    pub channel_name: String,
}

impl Default for Location {
    fn default() -> Self {
        Location {
            guild_name: Some("?".to_string()),
            channel_name: "?".to_string(),
        }
    }
}

impl Location {
    pub fn new(guild_name: Option<String>, channel: Channel) -> Self {
        let channel_name = match channel {
            Channel::Guild(guild_channel) => guild_channel.name,
            Channel::Private(private_channel) => private_channel.name(),
            _ => unimplemented!(),
        };

        Location {
            guild_name,
            channel_name,
        }
    }
}

pub struct LocationCache {
    data: Mutex<HashMap<ChannelId, Location>>,
}

impl LocationCache {
    pub fn new() -> Self {
        LocationCache {
            data: Mutex::new(HashMap::new()),
        }
    }

    async fn make_request(
        &self,
        channel_id: ChannelId,
        guild_id: Option<GuildId>,
        http: &Http,
    ) -> Result<Location, ()> {
        let guild_result = if let Some(id) = guild_id {
            Some(http.get_guild(id.0).await)
        } else {
            None
        };

        let channel_result = http.get_channel(channel_id.0).await;

        let location = match (guild_result, channel_result) {
            (Some(Ok(guild)), Ok(channel)) => Location::new(Some(guild.name), channel),
            (None, Ok(channel)) => Location::new(None, channel),
            _ => return Err(())
        };

        Ok(location)
    }

    pub async fn put(
        &self,
        channel_id: ChannelId,
        guild_id: Option<GuildId>,
        http: &Http,
    ) -> Result<Location, ()> {
        let mut channel_map = self.data.lock().await;

        match channel_map.entry(channel_id) {
            Entry::Vacant(entry) => {
                if let Ok(response) = self.make_request(channel_id, guild_id, http).await {
                    Ok(entry.insert(response).clone())
                } else {
                    Err(())
                }
            }
            Entry::Occupied(entry) => Ok(entry.get().clone()),
        }
    }

    pub async fn get(&self, channel_id: &ChannelId) -> Option<Location> {
        let channel_map = self.data.lock().await;
        channel_map.get(channel_id).cloned()
    }

    pub async fn get_or_fetch(
        &self,
        channel_id: ChannelId,
        guild_id: Option<GuildId>,
        http: &Http,
    ) -> Result<Location, ()> {
        if let Some(cached) = self.get(&channel_id).await {
            Ok(cached)
        } else {
            self.put(channel_id, guild_id, http).await
        }
    }
}
