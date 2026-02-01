use anyhow::Result;
use serde_json::json;

use crate::cache::Cache;
use crate::config::Config;
use crate::gcp::SecretManagerClient;

pub async fn execute(
    config: &Config,
    name: &str,
    env: &str,
    output: &str,
    no_cache: bool,
) -> Result<()> {
    // Try to get from cache first
    let mut cache = if !no_cache {
        Cache::load().ok()
    } else {
        None
    };

    let value = if let Some(cached) = cache.as_ref().and_then(|c| c.get(env, name)) {
        cached
    } else {
        let gcp_client = SecretManagerClient::new(config.clone()).await?;
        let value = gcp_client.get_secret(env, name).await?;

        // Update cache
        if !no_cache {
            if let Some(c) = cache.as_mut() {
                c.set(env, name, value.clone());
                let _ = c.save();
            } else if let Ok(mut c) = Cache::load() {
                // If cache failed to load earlier but we want to save now
                c.set(env, name, value.clone());
                let _ = c.save();
            }
        }
        value
    };

    match output {
        "json" => {
            let output = json!({
                "name": name,
                "environment": env,
                "value": value
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!("{}", value);
        }
    }

    Ok(())
}
