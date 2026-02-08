use anyhow::{anyhow, Error};
use tonic::Code;

pub fn map_gcp_error(err: Error) -> Error {
    // Try to downcast to tonic::Status
    if let Some(status) = err.downcast_ref::<tonic::Status>() {
        return match status.code() {
            Code::Unauthenticated => anyhow!(
                "Authentication failed.\n\
                 Run 'gcloud auth application-default login' to authenticate your local environment."
            ),
            Code::PermissionDenied => anyhow!(
                "Permission denied.\n\
                 Ensure your account has the 'Secret Manager Secret Accessor' (roles/secretmanager.secretAccessor) role for this project."
            ),
            Code::NotFound => anyhow!(
                "Resource not found.\n\
                 Check if the GCP project ID is correct and the secret exists."
            ),
            Code::AlreadyExists => anyhow!(
                "Resource already exists.\n\
                 You are trying to create a secret that is already present."
            ),
            Code::Unavailable => anyhow!(
                "Service unavailable.\n\
                 Google Cloud Secret Manager might be experiencing issues or you have connectivity problems."
            ),
            _ => anyhow!("Google Cloud Error: {}", status.message()),
        };
    }

    // If it's not a tonic::Status, just return the original error
    err
}
