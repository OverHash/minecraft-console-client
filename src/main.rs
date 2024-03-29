#![deny(clippy::pedantic)]
mod authentication;
mod cache;
mod config;
mod get_server_info;
mod protocol;

use std::io;

use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // get config and cache
    let config = config::get()?;

    // only read cache if enabled in config
    let fs_cache = config
        .cache_enabled
        .then(cache::Cache::get)
        .unwrap_or(Ok(None))?;
    let fs_cache_exists = fs_cache.is_some();
    let mut cache = fs_cache.unwrap_or_default();

    // get minecraft token
    let authenticate_cache = if fs_cache_exists { Some(&cache) } else { None };
    let authenticate_result =
        authentication::authenticate(&client, io::stdin().lock(), authenticate_cache).await?;
    let token = authenticate_result.minecraft_token;

    match authenticate_result.retrieve_type {
        authentication::RetrieveType::FromCache => (),
        authentication::RetrieveType::FromUserLogin {
            microsoft_refresh_token,
            expires_in,
        } => {
            if config.cache_enabled {
                // save to cache
                cache.save_minecraft_token(
                    token.clone(),
                    chrono::Utc::now() + chrono::Duration::seconds(i64::from(expires_in)),
                )?;
                cache.save_microsoft_refresh_token(microsoft_refresh_token)?;
            }
        }
    }

    println!("Got authentication token: {}", token);
    // retrieve server version
    get_server_info::get_server_info(config.server_url).await?;

    Ok(())
}
