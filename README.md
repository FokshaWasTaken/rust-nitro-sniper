[![status: archive](https://github.com/GIScience/badges/raw/master/status/archive.svg)]()

This project is currently ***discontinued***, we will continue providing help for the time being but there will be no work on it until further notice!

# =^.^= Rust Nitro Sniper

[![Build Status](https://travis-ci.org/Melonai/rust-nitro-sniper.svg?branch=master)](https://travis-ci.org/Melonai/rust-nitro-sniper)

Hi! **RNS** is a **simple** and **easy to use** Nitro Sniper for Discord written in the speedy Rust language.


# 📡 SNR
[Server Nitro Ranker](https://github.com/Melonai/SNR), or just [SNR](https://github.com/Melonai/SNR), is an auxiliary tool for RNS that helps you get a better understanding of the Nitro sniping value of the Discord servers you're currently in.


## Features

Feature-wise RNS can compete with other paid and closed-source snipers, but without any cost and risk and attached!

- Snipes on multiple accounts at once using sub-tokens

- Keeps your accounts within the rate limits using a cache of codes to not request again.

- Shows you comprehensive information about each attempted and successful snipe in the logs

- Discord webhook support

...and even more to come.

## Running

1. [Download](https://github.com/Melonai/rust-nitro-sniper/releases/) or build an executable for your respective platform.

2. Run the executable once in your directory of choice.

3. Change the default values as explained in [Configuration](https://github.com/Melonai/rust-nitro-sniper#configuration) in the `rns-config.json` config file created by the executable in your directory.

3. Run the executable again and enjoy!

## Configuration

The `rns-config.json` file created by RNS is formatted in this manner:

```json
{
  "main_token": "YOUR_TOKEN",
  "snipe_on_main_token": true,
  "sub_tokens": ["YOUR_SECOND_TOKEN", "...", "YOUR_NTH_TOKEN"],
  "webhook": "https://discordapp.com/api/webhooks/.../...",
  "guild_blacklist": [123456789123456789]
}
```
...where:

- ...the `main_token` property is the account that will receive the Nitro in the case that a snipe succeeds.

- ...the `snipe_on_main_token` property controls whether RNS tries connecting to your main account and tries to snipe a Nitro in your guilds.

- ...the `sub_tokens` property is a list of all the tokens with which RNS will try to snipe.

- ...the `webhook` is the Discord webhook URL. Leave blank if you don't need webhook messages.

- ...and the `guild_blacklist` is the list of Guild IDs you want RNS to ignore. 

---
#### Disclaimer

We must state that any usage of this tool with your personal user account token is a breach of the Discord Terms of Service, and we, the developers of RNS, will not take responsibility for any punishment (e.g. permanent ban) your account(s) could receive for using it.

/ᐠ｡ꞈ｡ᐟ\ **Thanks for reading!**

