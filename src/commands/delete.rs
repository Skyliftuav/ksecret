use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

use crate::config::Config;
use crate::gcp::SecretManagerClient;

pub async fn execute(config: &Config, name: &str, env: &str, force: bool) -> Result<()> {
    if !force {
        print!(
            "{} Are you sure you want to delete secret '{}' from environment '{}'? [y/N] ",
            "?".yellow().bold(),
            name.cyan(),
            env.cyan()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
    }

    let gcp_client = SecretManagerClient::new(config.clone()).await?;
    gcp_client.delete_secret(env, name).await?;

    println!(
        "{} Secret '{}' deleted from environment '{}'",
        "OK".green().bold(),
        name.cyan(),
        env.cyan()
    );

    Ok(())
}
