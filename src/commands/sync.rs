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

        // Determine if secret is multi-value (JSON/YAML) or single value
        let mut data = BTreeMap::new();
        let mut is_multi_value = false;

        // Try parsing as JSON Object first
        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(&value) {
            is_multi_value = true;
            for (k, v) in map {
                let v_str = match v {
                    serde_json::Value::String(s) => s,
                    _ => v.to_string(),
                };
                data.insert(k, v_str.into_bytes());
            }
        }
        // If not JSON object, try parsing as YAML Mapping
        else if let Ok(serde_yaml::Value::Mapping(map)) = serde_yaml::from_str::<serde_yaml::Value>(&value) {
            is_multi_value = true;
            for (k, v) in map {
                if let Some(k_str) = k.as_str() {
                    let v_str = match v {
                        serde_yaml::Value::String(s) => s,
                        serde_yaml::Value::Bool(b) => b.to_string(),
                        serde_yaml::Value::Number(n) => n.to_string(),
                        _ => {
                            // Serialize complex types back to string, trimming the newline usually added by to_string
                            serde_yaml::to_string(&v)
                                .unwrap_or_default()
                                .trim()
                                .to_string()
                        }
                    };
                    data.insert(k_str.to_string(), v_str.into_bytes());
                }
            }
        }

        // Fallback to single value if parsing failed or didn't yield a map
        if !is_multi_value || data.is_empty() {
            data.clear(); // Ensure empty if partial parse
            data.insert("value".to_string(), value.into_bytes());
        }

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
