use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::BTreeMap;

use crate::config::Config;
use crate::gcp::SecretManagerClient;
use crate::k8s::KubeClient;

pub async fn execute(
    config: &Config,
    environment: &str,
    namespace: Option<String>,
    context: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let namespace = namespace.unwrap_or_else(|| environment.to_string());

    println!(
        "{} Syncing secrets for environment '{}' to namespace '{}'",
        "->".blue().bold(),
        environment.cyan(),
        namespace.cyan()
    );

    if dry_run {
        println!("{}", "  (dry-run mode - no changes will be made)".yellow());
    }

    // Initialize clients
    let gcp_client = SecretManagerClient::new(config.clone())
        .await
        .context("Failed to initialize GCP client")?;

    let k8s_client = KubeClient::new(context.as_deref())
        .await
        .context("Failed to initialize Kubernetes client")?;

    // Check if namespace exists
    if !k8s_client.namespace_exists(&namespace).await? {
        anyhow::bail!("Namespace '{}' does not exist", namespace);
    }

    // List secrets from GCP
    let secrets = gcp_client.list_secrets(environment).await?;

    if secrets.is_empty() {
        println!(
            "{} No secrets found for environment '{}'",
            "!".yellow().bold(),
            environment
        );
        return Ok(());
    }

    println!("  Found {} secret(s) to sync", secrets.len().to_string().green());

    // Sync each secret
    for secret_info in &secrets {
        print!("  {} {}... ", "->".blue(), secret_info.name);

        if dry_run {
            println!("{}", "skipped (dry-run)".yellow());
            continue;
        }

        // Get secret value from GCP
        let value = gcp_client
            .get_secret(environment, &secret_info.name)
            .await
            .with_context(|| format!("Failed to get secret: {}", secret_info.name))?;

        // Create data map with "value" as the key
        let mut data = BTreeMap::new();
        data.insert("value".to_string(), value.into_bytes());

        // Apply to Kubernetes
        k8s_client
            .apply_secret(&namespace, &secret_info.name, data)
            .await
            .with_context(|| format!("Failed to apply secret: {}", secret_info.name))?;

        println!("{}", "done".green());
    }

    println!(
        "\n{} Successfully synced {} secret(s) to namespace '{}'",
        "OK".green().bold(),
        secrets.len(),
        namespace.cyan()
    );

    Ok(())
}
