use anyhow::{anyhow, Error};
use kube::Error as KubeError;

pub fn map_k8s_error(err: Error) -> Error {
    if let Some(kube_err) = err.downcast_ref::<KubeError>() {
        return match kube_err {
            KubeError::Api(api_err) => {
                match api_err.code {
                    401 => anyhow!("Kubernetes Authentication failed.\nCheck your kubeconfig credentials."),
                    403 => anyhow!("Kubernetes Permission denied.\nYou don't have permission to perform this action in the namespace."),
                    404 => anyhow!("Kubernetes Resource not found."),
                    _ => anyhow!("Kubernetes API Error: {}", api_err.message),
                }
            }
            // Kubeconfig error is likely wrapped in a different variant or needs to be matched differently
            // Looking at kube-rs docs for v0.98, the error enum structure might have changed or I was using old variants
            // Let's use a catch-all for now or just the display impl if specific variants are not found
            _ => anyhow!("Kubernetes Error: {}", kube_err),
        };
    }
    err
}
