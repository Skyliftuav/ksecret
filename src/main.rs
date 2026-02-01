use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;
mod config;
mod gcp;
mod k8s;
mod cache;

/// ksecret - Kubernetes Secrets Management Tool
///
/// Manage environment secrets for Kubernetes clusters using Google Cloud Secret Manager.
/// Secrets are organized by environment (dev, staging, prod) and can be synced to any
/// Kubernetes cluster with a single command.
#[derive(Parser)]
#[command(name = "ksecret")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Google Cloud Project ID (overrides config file)
    #[arg(long, env = "KSECRET_GCP_PROJECT")]
    project: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync all secrets for an environment to a Kubernetes namespace
    Sync {
        /// Environment name (e.g., dev, staging, prod)
        #[arg(value_name = "ENV")]
        environment: String,

        /// Target Kubernetes namespace (defaults to environment name)
        #[arg(short, long)]
        namespace: Option<String>,

        /// Kubernetes context to use (defaults to current context)
        #[arg(short, long)]
        context: Option<String>,

        /// Perform a dry run without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Get a secret value from Google Cloud Secret Manager
    Get {
        /// Secret name
        #[arg(value_name = "NAME")]
        name: String,

        /// Environment name
        #[arg(short, long, required = true)]
        env: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        output: String,

        /// Skip cache and fetch directly from GCP
        #[arg(long)]
        no_cache: bool,
    },

    /// Set a secret value in Google Cloud Secret Manager
    Set {
        /// Secret name
        #[arg(value_name = "NAME")]
        name: String,

        /// Environment name
        #[arg(short, long, required = true)]
        env: String,

        /// Secret value (will prompt if not provided)
        #[arg(long)]
        value: Option<String>,

        /// Read value from stdin
        #[arg(long)]
        stdin: bool,
    },

    /// List all secrets for an environment
    List {
        /// Environment name
        #[arg(short, long, required = true)]
        env: String,

        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        output: String,
    },

    /// Delete a secret from Google Cloud Secret Manager
    Delete {
        /// Secret name
        #[arg(value_name = "NAME")]
        name: String,

        /// Environment name
        #[arg(short, long, required = true)]
        env: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Initialize configuration file
    Init {
        /// Google Cloud Project ID
        #[arg(long, required = true)]
        project: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "ksecret=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Install default crypto provider for rustls
    let _ = rustls::crypto::ring::default_provider().install_default();

    let cli = Cli::parse();

    // Execute command
    let result = match cli.command {
        Commands::Sync {
            environment,
            namespace,
            context,
            dry_run,
        } => {
            let config = config::Config::load(cli.project)?;
            commands::sync::execute(&config, &environment, namespace, context, dry_run).await
        }
        Commands::Get { name, env, output, no_cache } => {
            let config = config::Config::load(cli.project)?;
            commands::get::execute(&config, &name, &env, &output, no_cache).await
        }
        Commands::Set {
            name,
            env,
            value,
            stdin,
        } => {
            let config = config::Config::load(cli.project)?;
            commands::set::execute(&config, &name, &env, value, stdin).await
        }
        Commands::List { env, output } => {
            let config = config::Config::load(cli.project)?;
            commands::list::execute(&config, &env, &output).await
        }
        Commands::Delete { name, env, force } => {
            let config = config::Config::load(cli.project)?;
            commands::delete::execute(&config, &name, &env, force).await
        }
        Commands::Init { project } => commands::init::execute(&project).await,
    };

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{} {:#}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}
