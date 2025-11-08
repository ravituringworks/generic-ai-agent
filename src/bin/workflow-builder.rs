//! Workflow Builder - Desktop Application
//!
//! A native desktop application for creating and managing complex workflows,
//! built with Tauri for a native experience with web-based UI.

#[cfg(feature = "tauri")]
use std::process::{Command, Stdio};
#[cfg(feature = "tauri")]
use std::time::Duration;
#[cfg(feature = "tauri")]
use tauri::Manager;

// Agency daemon URL
#[cfg(feature = "tauri")]
const DAEMON_URL: &str = "http://localhost:8080";
#[cfg(feature = "tauri")]
const DAEMON_HEALTH_URL: &str = "http://localhost:8080/health";

#[cfg(feature = "tauri")]
/// Check if daemon is running and spawn it if not
async fn ensure_daemon_running() -> Result<(), String> {
    // First check if daemon is already running
    let client = reqwest::Client::new();
    match client
        .get(DAEMON_HEALTH_URL)
        .timeout(Duration::from_secs(2))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            println!("✓ Daemon is already running");
            return Ok(());
        }
        _ => {
            println!("Daemon not running, spawning...");
        }
    }

    // Find the cargo binary location
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    // Get the project root directory (where Cargo.toml is)
    let project_root = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or_else(|| "Failed to determine project root".to_string())?
        .to_path_buf();

    println!("Spawning daemon from: {:?}", project_root);

    // Spawn daemon process in background
    Command::new(cargo)
        .args(["run", "--bin", "agency-daemon", "--release"])
        .current_dir(&project_root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn daemon: {}", e))?;

    // Wait for daemon to be ready (up to 30 seconds)
    print!("Waiting for daemon to start");
    for i in 0..30 {
        tokio::time::sleep(Duration::from_secs(1)).await;

        match client
            .get(DAEMON_HEALTH_URL)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                println!("\n✓ Daemon is ready after {} seconds", i + 1);
                return Ok(());
            }
            _ => {
                print!(".");
                use std::io::Write;
                std::io::stdout().flush().ok();
            }
        }
    }

    Err("Daemon failed to start within 30 seconds".to_string())
}

#[cfg(feature = "tauri")]
fn main() {
    println!("Starting The Agency Workflow Builder...");

    // Check and spawn daemon before starting Tauri
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let daemon_result = runtime.block_on(async { ensure_daemon_running().await });

    match daemon_result {
        Ok(_) => {
            println!("✓ Daemon is ready");
            println!("✓ Workflow UI available at: {}/workflow-ui", DAEMON_URL);
        }
        Err(e) => {
            eprintln!("Warning: Failed to ensure daemon is running: {}", e);
            eprintln!("You may need to start the daemon manually with: cargo run --bin agency-daemon");
            eprintln!("Press Enter to continue anyway or Ctrl+C to exit...");
            std::io::stdin().read_line(&mut String::new()).ok();
        }
    }

    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // Load the workflow UI from the daemon
            let url = format!("{}/workflow-ui", DAEMON_URL);
            window
                .eval(&format!(r#"window.location.href = "{}";"#, url))
                .ok();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(not(feature = "tauri"))]
fn main() {
    println!("Tauri feature not enabled. Use --features tauri to build the desktop app.");
    println!("To build the desktop app, run: cargo build --bin workflow-builder --features tauri");
}
