use crate::config::Config;
use anyhow::{Context, Result};
use google_cloud_secretmanager_v1::client::SecretManagerService;

/// Wrapper around Google Cloud Secret Manager client
pub struct SecretManagerClient {
    client: SecretManagerService,
    config: Config,
}

/// Represents a secret retrieved from GCP
#[derive(Debug, Clone)]
pub struct SecretInfo {
    pub name: String,
    pub environment: String,
    pub created_at: Option<String>,
}

impl SecretManagerClient {
    /// Create a new Secret Manager client
    pub async fn new(config: Config) -> Result<Self> {
        let client = SecretManagerService::builder()
            .build()
            .await
            .context("Failed to create Secret Manager client")?;

        Ok(Self { client, config })
    }

    /// List all secrets for a given environment
    pub async fn list_secrets(&self, environment: &str) -> Result<Vec<SecretInfo>> {
        let parent = format!("projects/{}", self.config.gcp_project_id);
        let prefix = format!("{}-{}-", self.config.secret_prefix, environment);

        let mut secrets = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut request = self.client.list_secrets().set_parent(&parent);

            if let Some(token) = &page_token {
                request = request.set_page_token(token);
            }

            let response = request.send().await.context("Failed to list secrets")?;

            for secret in response.secrets.iter() {
                // Extract the secret name from the full resource path
                let full_name = secret.name.as_str();
                let short_name = full_name.rsplit('/').next().unwrap_or(full_name);

                // Filter by environment prefix
                if short_name.starts_with(&prefix) {
                    let secret_name = short_name.strip_prefix(&prefix).unwrap_or(short_name);
                    secrets.push(SecretInfo {
                        name: secret_name.to_string(),
                        environment: environment.to_string(),
                        created_at: secret.create_time.as_ref().map(|t| {
                            use chrono::{DateTime, Utc};
                            let dt = DateTime::<Utc>::from_timestamp(t.seconds(), t.nanos() as u32);
                            match dt {
                                Some(d) => d.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                                None => "Unknown".to_string(),
                            }
                        }),
                    });
                }
            }

            // Check for more pages
            if response.next_page_token.is_empty() {
                break;
            }
            page_token = Some(response.next_page_token.clone());
        }

        Ok(secrets)
    }

    /// Get a secret value
    pub async fn get_secret(&self, environment: &str, name: &str) -> Result<String> {
        let version_name = self.config.build_version_name(environment, name, "latest");

        let response = self
            .client
            .access_secret_version()
            .set_name(&version_name)
            .send()
            .await
            .with_context(|| format!("Failed to access secret: {}", name))?;

        let payload = response.payload.context("Secret has no payload")?;

        let data =
            String::from_utf8(payload.data.to_vec()).context("Secret data is not valid UTF-8")?;

        Ok(data)
    }

    /// Create or update a secret
    pub async fn set_secret(&self, environment: &str, name: &str, value: &str) -> Result<()> {
        let secret_id = self.config.build_secret_name(environment, name);
        let parent = format!("projects/{}", self.config.gcp_project_id);
        let secret_name = format!("{}/secrets/{}", parent, secret_id);

        // Try to get the secret first to see if it exists
        let secret_exists = self
            .client
            .get_secret()
            .set_name(&secret_name)
            .send()
            .await
            .is_ok();

        if !secret_exists {
            let mut replication = google_cloud_secretmanager_v1::model::Replication::default();
            replication.replication = Some(
                google_cloud_secretmanager_v1::model::replication::Replication::Automatic(
                    Box::new(
                        google_cloud_secretmanager_v1::model::replication::Automatic::default(),
                    ),
                ),
            );

            let mut secret = google_cloud_secretmanager_v1::model::Secret::default();
            secret.replication = Some(replication);

            // Create the secret
            self.client
                .create_secret()
                .set_parent(&parent)
                .set_secret_id(&secret_id)
                .set_secret(secret)
                .send()
                .await
                .with_context(|| format!("Failed to create secret: {}", name))?;
        }

        let mut payload = google_cloud_secretmanager_v1::model::SecretPayload::default();
        payload.data = value.as_bytes().to_vec().into();

        // Add a new version with the secret data
        self.client
            .add_secret_version()
            .set_parent(&secret_name)
            .set_payload(payload)
            .send()
            .await
            .with_context(|| format!("Failed to add secret version: {}", name))?;

        Ok(())
    }

    /// Delete a secret
    pub async fn delete_secret(&self, environment: &str, name: &str) -> Result<()> {
        let secret_name = self.config.build_resource_name(environment, name);

        self.client
            .delete_secret()
            .set_name(&secret_name)
            .send()
            .await
            .with_context(|| format!("Failed to delete secret: {}", name))?;

        Ok(())
    }
}
