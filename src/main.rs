use arboretum::config::Config;
use arboretum::db::Database;
use arboretum::server::{create_router, spawn_cleanup_task};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "arboretum")]
#[command(about = "OpenTelemetry log and trace caching server", long_about = None)]
struct Args {
    /// Path to configuration file (TOML)
    #[arg(short, long, value_name = "FILE", global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Start the server (default)
    Serve {
        /// Enable systemd socket activation
        #[arg(short, long)]
        systemd: bool,
    },

    #[cfg(feature = "tui")]
    /// Run the TUI to view logs and traces
    Tui {
        /// Server URL to connect to
        #[arg(
            short,
            long,
            value_name = "URL",
            default_value = "http://localhost:3333"
        )]
        url: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut systemd = false;
    match args.command {
        #[cfg(feature = "tui")]
        Some(Command::Tui { url }) => {
            // For TUI, we don't want tracing output to interfere
            arboretum::tui::run(url).await?;
        }
        Some(Command::Serve { systemd: false }) | None => {}
        Some(Command::Serve { systemd: true }) => {
            systemd = true;
        }
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "arboretum=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = if let Some(config_path) = args.config {
        info!("Loading configuration from file: {:?}", config_path);
        Config::from_file(config_path)?.with_env_overrides()
    } else {
        info!("No configuration file provided, using environment variables and defaults");
        Config::from_env()
    };

    info!("Arboretum starting...");
    info!("Configuration:");
    if !systemd {
        info!("  Listen address: {}", config.listen_addr);
    }
    info!("  Database path: {:?}", config.db_path);
    info!("  Max retention: {} seconds", config.max_retention_seconds);
    info!(
        "  Cleanup interval: {} seconds",
        config.cleanup_interval_seconds
    );

    let db = Database::new(&config.db_path).await?;

    let app = create_router(db.clone());

    let _cleanup_handle = spawn_cleanup_task(db, &config);

    let listener = if systemd {
        let socket = systemd_connector::sockets()?
            .into_iter()
            .find(|socket| socket.name() == Some("arboretum"))
            .ok_or("systemd socket not found")?;
        info!("Listening on systemd://arboretum");

        tokio::net::TcpListener::from_std(socket.listener()?)?
    } else {
        info!("Listening on {}", config.listen_addr);

        tokio::net::TcpListener::bind(&config.listen_addr).await?
    };
    axum::serve(listener, app).await?;

    Ok(())
}
