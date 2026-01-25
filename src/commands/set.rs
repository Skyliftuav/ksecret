use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Read};

use crate::config::Config;
use crate::gcp::SecretManagerClient;

pub async fn execute(
    config: &Config,
    name: &str,
    env: &str,
    value: Option<String>,
    stdin: bool,
) -> Result<()> {
    let secret_value = if stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        buffer.trim_end().to_string()
    } else if let Some(v) = value {
        v
    } else {
        print!("Enter secret value: ");
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .context("Failed to read input")?;
        buffer.trim_end().to_string()
    };

    let gcp_client = SecretManagerClient::new(config.clone()).await?;
    gcp_client.set_secret(env, name, &secret_value).await?;

    println!(
        "{} Secret '{}' set for environment '{}'",
        "OK".green().bold(),
        name.cyan(),
        env.cyan()
    );

    Ok(())
}
