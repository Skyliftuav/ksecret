use anyhow::Result;
use colored::Colorize;
use serde_json::json;

use crate::config::Config;
use crate::gcp::SecretManagerClient;

pub async fn execute(config: &Config, env: &str, output: &str) -> Result<()> {
    let gcp_client = SecretManagerClient::new(config.clone()).await?;
    let secrets = gcp_client.list_secrets(env).await?;

    match output {
        "json" => {
            let output: Vec<_> = secrets
                .iter()
                .map(|s| {
                    json!({
                        "name": s.name,
                        "environment": s.environment,
                        "created_at": s.created_at
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            if secrets.is_empty() {
                println!(
                    "{} No secrets found for environment '{}'",
                    "!".yellow().bold(),
                    env
                );
                return Ok(());
            }

            println!(
                "{} Secrets for environment '{}':\n",
                "->".blue().bold(),
                env.cyan()
            );

            println!("  {:<30} {:<20}", "NAME".bold(), "CREATED".bold());
            println!("  {}", "-".repeat(50));

            for secret in &secrets {
                let created = secret
                    .created_at
                    .as_deref()
                    .unwrap_or("-")
                    .chars()
                    .take(19)
                    .collect::<String>();
                println!("  {:<30} {:<20}", secret.name, created);
            }

            println!("\n  Total: {} secret(s)", secrets.len().to_string().green());
        }
    }

    Ok(())
}
