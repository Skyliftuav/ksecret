use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Google Cloud Project ID
    pub gcp_project_id: String,

    /// Secret name prefix (default: "k8s")
    #[serde(default = "default_prefix")]
    pub secret_prefix: String,
}

fn default_prefix() -> String {
    "k8s".to_string()
}

impl Config {
    /// Load configuration from file and environment
    pub fn load(project_override: Option<String>) -> Result<Self> {
        let config_path = Self::config_path()?;

        let mut config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {:?}", config_path))?
        } else {
            // Return error if no config and no override
            if project_override.is_none() {
                anyhow::bail!(
                    "No configuration found. Run 'ksecret init --project <PROJECT_ID>' to initialize."
                );
            }
            Config {
                gcp_project_id: String::new(),
                secret_prefix: default_prefix(),
            }
        };

        // Apply override if provided
        if let Some(project) = project_override {
            config.gcp_project_id = project;
        }

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create parent directories if needed
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        std::fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }

    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        if let Ok(path) = std::env::var("KSECRET_CONFIG_FILE") {
            return Ok(PathBuf::from(path));
        }
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let config_dir = home.join(".config").join("ksecret");
        Ok(config_dir.join("config.toml"))
    }

    /// Build the full secret name for GCP Secret Manager
    pub fn build_secret_name(&self, environment: &str, name: &str) -> String {
        format!("{}-{}-{}", self.secret_prefix, environment, name)
    }

    /// Parse environment and name from a full GCP secret name
    #[allow(dead_code)]
    pub fn parse_secret_name(&self, full_name: &str) -> Option<(String, String)> {
        let prefix = format!("{}-", self.secret_prefix);
        if !full_name.starts_with(&prefix) {
            return None;
        }

        let remainder = &full_name[prefix.len()..];
        let parts: Vec<&str> = remainder.splitn(2, '-').collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }

    /// Build the GCP Secret Manager resource name
    pub fn build_resource_name(&self, environment: &str, name: &str) -> String {
        format!(
            "projects/{}/secrets/{}",
            self.gcp_project_id,
            self.build_secret_name(environment, name)
        )
    }

    /// Build the GCP Secret Manager version resource name
    pub fn build_version_name(&self, environment: &str, name: &str, version: &str) -> String {
        format!(
            "projects/{}/secrets/{}/versions/{}",
            self.gcp_project_id,
            self.build_secret_name(environment, name),
            version
        )
    }
}
