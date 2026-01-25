use anyhow::Result;
use colored::Colorize;
use crate::config::Config;

pub async fn execute(project: &str) -> Result<()> {
    let config = Config {
        gcp_project_id: project.to_string(),
        secret_prefix: "k8s".to_string(),
    };

    config.save()?;

    let config_path = Config::config_path()?;
    println!(
        "{} Configuration saved to {}",
        "✓".green().bold(),
        config_path.display()
    );
    println!("  GCP Project ID: {}", project.cyan());

    Ok(())
}
