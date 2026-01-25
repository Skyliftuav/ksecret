use anyhow::Result;
use serde_json::json;

use crate::config::Config;
use crate::gcp::SecretManagerClient;

pub async fn execute(config: &Config, name: &str, env: &str, output: &str) -> Result<()> {
    let gcp_client = SecretManagerClient::new(config.clone()).await?;
    let value = gcp_client.get_secret(env, name).await?;

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
