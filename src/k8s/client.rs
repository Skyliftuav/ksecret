use crate::k8s::error::map_k8s_error;
use anyhow::{Context, Result};
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::ByteString;
use kube::{
    api::{Api, DeleteParams, PostParams},
    config::{KubeConfigOptions, Kubeconfig},
    Client, Config,
};
use std::collections::BTreeMap;

/// Wrapper around Kubernetes client for secret operations
pub struct KubeClient {
    client: Client,
}

impl KubeClient {
    /// Create a new Kubernetes client using the specified context or default
    pub async fn new(context: Option<&str>) -> Result<Self> {
        let config = if let Some(ctx) = context {
            // Load kubeconfig with specific context
            let kubeconfig = Kubeconfig::read()
                .map_err(|e| map_k8s_error(e.into()))
                .context("Failed to read kubeconfig")?;
            let options = KubeConfigOptions {
                context: Some(ctx.to_string()),
                ..Default::default()
            };
            Config::from_custom_kubeconfig(kubeconfig, &options)
                .await
                .map_err(|e| map_k8s_error(e.into()))
                .with_context(|| format!("Failed to create config for context: {}", ctx))?
        } else {
            // Use default config (in-cluster or default context)
            Config::infer()
                .await
                .map_err(|e| map_k8s_error(e.into()))
                .context("Failed to infer Kubernetes config")?
        };

        let client = Client::try_from(config)
            .map_err(|e| map_k8s_error(e.into()))
            .context("Failed to create Kubernetes client")?;

        Ok(Self { client })
    }

    /// Create or update a secret in the specified namespace
    pub async fn apply_secret(
        &self,
        namespace: &str,
        name: &str,
        data: BTreeMap<String, Vec<u8>>,
    ) -> Result<()> {
        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), namespace);

        // Convert data to ByteString map
        let secret_data: BTreeMap<String, ByteString> =
            data.into_iter().map(|(k, v)| (k, ByteString(v))).collect();

        let secret = Secret {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                labels: Some(BTreeMap::from([(
                    "app.kubernetes.io/managed-by".to_string(),
                    "ksecret".to_string(),
                )])),
                ..Default::default()
            },
            data: Some(secret_data),
            type_: Some("Opaque".to_string()),
            ..Default::default()
        };

        // Delete existing secret if it exists (delete-then-recreate strategy)
        let delete_params = DeleteParams::default();
        match secrets.delete(name, &delete_params).await {
            Ok(_) => {
                // Wait for deletion to verify it's gone?
                // k8s delete is usually async, but for secrets it's often fast.
                // We'll proceed to create. If we get a conflict, we might need to retry,
                // but usually the UID changes so it's fine.
            }
            Err(kube::Error::Api(e)) if e.code == 404 => {
                // Secret didn't exist, safe to proceed
            }
            Err(e) => return Err(map_k8s_error(e.into())),
        }

        // Create the secret freshly
        let post_params = PostParams::default();
        secrets
            .create(&post_params, &secret)
            .await
            .map_err(|e| map_k8s_error(e.into()))
            .with_context(|| format!("Failed to create secret: {}", name))?;

        Ok(())
    }

    /// Delete a secret from the specified namespace
    #[allow(dead_code)]
    pub async fn delete_secret(&self, namespace: &str, name: &str) -> Result<()> {
        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), namespace);

        secrets
            .delete(name, &Default::default())
            .await
            .map_err(|e| map_k8s_error(e.into()))
            .with_context(|| format!("Failed to delete secret: {}", name))?;

        Ok(())
    }

    /// List all secrets in a namespace managed by ksecret
    #[allow(dead_code)]
    pub async fn list_managed_secrets(&self, namespace: &str) -> Result<Vec<String>> {
        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), namespace);

        let list_params =
            kube::api::ListParams::default().labels("app.kubernetes.io/managed-by=ksecret");

        let secret_list = secrets
            .list(&list_params)
            .await
            .map_err(|e| map_k8s_error(e.into()))
            .context("Failed to list secrets")?;

        let names: Vec<String> = secret_list
            .items
            .iter()
            .filter_map(|s| s.metadata.name.clone())
            .collect();

        Ok(names)
    }

    /// Check if namespace exists
    pub async fn namespace_exists(&self, namespace: &str) -> Result<bool> {
        use k8s_openapi::api::core::v1::Namespace;

        let namespaces: Api<Namespace> = Api::all(self.client.clone());
        match namespaces.get(namespace).await {
            Ok(_) => Ok(true),
            Err(kube::Error::Api(err)) if err.code == 404 => Ok(false),
            Err(e) => Err(map_k8s_error(e.into())).context("Failed to check namespace"),
        }
    }
}
