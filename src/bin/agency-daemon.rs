//! Agency Daemon - Long-running service for the agent platform
//!
//! This daemon provides:
//! - REST API server for agent operations
//! - Workflow orchestration and management
//! - Long-running workflow execution with suspend/resume
//! - Background task processing
//!
//! Usage:
//!   agency-daemon [OPTIONS]
//!
//! Options:
//!   --config <PATH>      Path to configuration file (default: config.toml)
//!   --host <HOST>        API server host (default: 127.0.0.1)
//!   --port <PORT>        API server port (default: 8080)
//!   --daemon             Run as background daemon (Unix only)
//!   --pid-file <PATH>    PID file path for daemon mode
//!   --log-file <PATH>    Log file path for daemon mode

use std::fs::File;
use std::path::PathBuf;
use the_agency::api::{start_server, AppState};
use the_agency::config::AgentConfig;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Debug)]
struct DaemonConfig {
    config_path: PathBuf,
    host: String,
    port: u16,
    daemon_mode: bool,
    pid_file: Option<PathBuf>,
    log_file: Option<PathBuf>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from("config.toml"),
            host: "127.0.0.1".to_string(),
            port: 8080,
            daemon_mode: false,
            pid_file: None,
            log_file: None,
        }
    }
}

fn parse_args() -> DaemonConfig {
    let mut config = DaemonConfig::default();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                if let Some(path) = args.next() {
                    config.config_path = PathBuf::from(path);
                }
            }
            "--host" => {
                if let Some(host) = args.next() {
                    config.host = host;
                }
            }
            "--port" => {
                if let Some(port) = args.next() {
                    if let Ok(p) = port.parse() {
                        config.port = p;
                    }
                }
            }
            "--daemon" => {
                config.daemon_mode = true;
            }
            "--pid-file" => {
                if let Some(path) = args.next() {
                    config.pid_file = Some(PathBuf::from(path));
                }
            }
            "--log-file" => {
                if let Some(path) = args.next() {
                    config.log_file = Some(PathBuf::from(path));
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                print_help();
                std::process::exit(1);
            }
        }
    }

    config
}

fn print_help() {
    println!("Agency Daemon - Long-running service for the agent platform");
    println!();
    println!("USAGE:");
    println!("    agency-daemon [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --config <PATH>      Path to configuration file (default: config.toml)");
    println!("    --host <HOST>        API server host (default: 127.0.0.1)");
    println!("    --port <PORT>        API server port (default: 8080)");
    println!("    --daemon             Run as background daemon (Unix only)");
    println!("    --pid-file <PATH>    PID file path for daemon mode");
    println!("    --log-file <PATH>    Log file path for daemon mode");
    println!("    --help, -h           Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Run in foreground");
    println!("    agency-daemon --config config.toml --port 8080");
    println!();
    println!("    # Run as daemon");
    println!(
        "    agency-daemon --daemon --pid-file /var/run/agency.pid --log-file /var/log/agency.log"
    );
}

fn setup_logging(log_file: Option<PathBuf>) -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,the_agency=debug"));

    if let Some(log_path) = log_file {
        let file = File::create(log_path)?;
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_writer(file))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer())
            .init();
    }

    Ok(())
}

#[cfg(unix)]
fn daemonize_process(pid_file: Option<PathBuf>) -> anyhow::Result<()> {
    use daemonize::Daemonize;

    let mut daemon = Daemonize::new();

    if let Some(pid_path) = pid_file {
        daemon = daemon.pid_file(pid_path);
    }

    daemon.start()?;
    Ok(())
}

#[cfg(not(unix))]
fn daemonize_process(_pid_file: Option<PathBuf>) -> anyhow::Result<()> {
    Err(anyhow::anyhow!(
        "Daemon mode is only supported on Unix systems"
    ))
}

async fn run_server(config: DaemonConfig) -> anyhow::Result<()> {
    info!("Loading agent configuration from: {:?}", config.config_path);

    // Load agent configuration
    let agent_config = if config.config_path.exists() {
        AgentConfig::from_file(&config.config_path)?
    } else {
        info!("Config file not found, using default configuration");
        AgentConfig::default()
    };

    // Create application state
    info!("Initializing agent and workflow engine...");
    let app_state = AppState::new(agent_config).await?;

    info!("Agency daemon starting...");
    info!("API server will listen on {}:{}", config.host, config.port);

    // Setup graceful shutdown
    let (tx, mut rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Shutdown signal received");
        tx.send(()).ok();
    });

    // Start the API server
    tokio::select! {
        result = start_server(app_state, &config.host, config.port) => {
            if let Err(e) = result {
                error!("Server error: {}", e);
                return Err(e.into());
            }
        }
        _ = &mut rx => {
            info!("Graceful shutdown initiated");
        }
    }

    info!("Agency daemon stopped");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = parse_args();

    // Setup logging
    setup_logging(config.log_file.clone())?;

    info!("Agency Daemon v{}", the_agency::VERSION);

    // Daemonize if requested
    if config.daemon_mode {
        #[cfg(unix)]
        {
            info!("Daemonizing process...");
            daemonize_process(config.pid_file.clone())?;
            // Re-setup logging after daemonization
            setup_logging(config.log_file.clone())?;
        }
        #[cfg(not(unix))]
        {
            error!("Daemon mode is only supported on Unix systems");
            return Err(anyhow::anyhow!(
                "Daemon mode not supported on this platform"
            ));
        }
    }

    // Run the server
    run_server(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DaemonConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(!config.daemon_mode);
    }
}
