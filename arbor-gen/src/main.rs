use clap::Parser;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use rand::Rng;
use std::time::Duration;
use tokio::signal;
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser, Debug)]
#[command(name = "arbor-gen")]
#[command(about = "Generate sample traces and logs for Arboretum")]
struct Args {
    /// OTLP endpoint URL
    #[arg(short, long, envvar = "ARBORETUM_LISTEN_ADDR")]
    endpoint: String,

    /// Service name for traces
    #[arg(short, long, default_value = "arbor-gen")]
    service: String,

    /// Delay between iterations in milliseconds
    #[arg(short, long, default_value = "1000")]
    delay: u64,

    /// Disable console logging (only export to OpenTelemetry)
    #[arg(short, long)]
    quiet: bool,
}

fn init_tracer(
    endpoint: &str,
    service_name: &str,
) -> Result<TracerProvider, Box<dyn std::error::Error>> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/traces", endpoint))
        .build()?;

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(Resource::new(vec![KeyValue::new(
            SERVICE_NAME,
            service_name.to_string(),
        )]))
        .build();

    global::set_tracer_provider(provider.clone());

    Ok(provider)
}

#[instrument]
async fn simulate_database_query(query: &str) -> Result<u32, String> {
    info!(query = %query, "Executing database query");

    let delay = rand::thread_rng().gen_range(10..100);
    tokio::time::sleep(Duration::from_millis(delay)).await;

    // Simulate occasional errors
    if rand::thread_rng().gen_bool(0.1) {
        error!(query = %query, "Database query failed");
        return Err("Connection timeout".to_string());
    }

    let rows = rand::thread_rng().gen_range(0..1000);
    debug!(rows = %rows, "Query returned rows");

    Ok(rows)
}

#[instrument]
async fn process_user_data(user_id: u64) -> Result<(), String> {
    info!(user_id = %user_id, "Processing user data");

    let rows =
        simulate_database_query(&format!("SELECT * FROM users WHERE id = {}", user_id)).await?;

    if rows == 0 {
        warn!(user_id = %user_id, "User not found");
        return Err("User not found".to_string());
    }

    tokio::time::sleep(Duration::from_millis(20)).await;
    debug!("User data processed successfully");

    Ok(())
}

#[instrument]
async fn fetch_related_items(user_id: u64, category: &str) -> Result<Vec<String>, String> {
    info!(user_id = %user_id, category = %category, "Fetching related items");

    let rows = simulate_database_query(&format!(
        "SELECT * FROM items WHERE user_id = {} AND category = '{}'",
        user_id, category
    ))
    .await?;

    let items: Vec<String> = (0..rows.min(5))
        .map(|i| format!("item-{}-{}", category, i))
        .collect();

    debug!(count = %items.len(), "Fetched items");

    Ok(items)
}

#[instrument]
async fn calculate_metrics(user_id: u64) -> u32 {
    info!(user_id = %user_id, "Calculating user metrics");

    tokio::time::sleep(Duration::from_millis(50)).await;

    let score = rand::thread_rng().gen_range(0..100);
    debug!(score = %score, "Calculated user score");

    score
}

#[instrument]
async fn handle_user_request(user_id: u64) -> Result<(), String> {
    info!(user_id = %user_id, "Handling user request");

    // Process user data
    if let Err(e) = process_user_data(user_id).await {
        error!(error = %e, "Failed to process user data");
        return Err(e);
    }

    // Fetch related items in parallel
    let categories = vec!["books", "movies", "music"];
    let mut tasks = vec![];

    for category in categories {
        let category = category.to_string();
        tasks.push(tokio::spawn(async move {
            fetch_related_items(user_id, &category).await
        }));
    }

    // Wait for all tasks
    for task in tasks {
        match task.await {
            Ok(Ok(items)) => {
                debug!(count = %items.len(), "Fetched category items");
            }
            Ok(Err(e)) => {
                warn!(error = %e, "Failed to fetch items");
            }
            Err(e) => {
                error!(error = %e, "Task panicked");
            }
        }
    }

    // Calculate metrics
    let score = calculate_metrics(user_id).await;
    info!(score = %score, "Request completed");

    Ok(())
}

#[instrument]
async fn simulate_background_job() {
    info!("Running background job");

    tokio::time::sleep(Duration::from_millis(100)).await;

    match simulate_database_query("DELETE FROM expired_sessions").await {
        Ok(rows) => info!(rows = %rows, "Cleaned up expired sessions"),
        Err(e) => error!(error = %e, "Failed to clean up sessions"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize tracing with OpenTelemetry
    let provider = init_tracer(&args.endpoint, &args.service)?;

    let telemetry = tracing_opentelemetry::layer()
        .with_level(true)
        .with_tracer(provider.tracer("arbor-gen"));

    // Conditionally add console logging based on --quiet flag
    let fmt_layer = if args.quiet {
        None
    } else {
        Some(tracing_subscriber::fmt::layer())
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "arbor_gen=debug,info".into()),
        )
        .with(fmt_layer)
        .with(telemetry)
        .init();

    info!(
        endpoint = %args.endpoint,
        service = %args.service,
        "Starting trace generator (press Ctrl+C to stop)"
    );

    let mut iteration = 0u64;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Received interrupt signal, shutting down");
                break;
            }
            _ = async {
                info!(iteration = %iteration, "Starting iteration");

                // Simulate multiple user requests
                let user_ids: Vec<u64> = (0..3)
                    .map(|_| rand::thread_rng().gen_range(1..1000))
                    .collect();

                let mut tasks = vec![];
                for user_id in user_ids {
                    tasks.push(tokio::spawn(
                        async move { handle_user_request(user_id).await },
                    ));
                }

                // Also run a background job
                tokio::spawn(simulate_background_job());

                // Wait for user requests to complete
                for task in tasks {
                    match task.await {
                        Ok(Ok(())) => debug!("User request completed"),
                        Ok(Err(e)) => warn!(error = %e, "User request failed"),
                        Err(e) => error!(error = %e, "Task panicked"),
                    }
                }

                iteration += 1;
                tokio::time::sleep(Duration::from_millis(args.delay)).await;
            } => {}
        }
    }

    // Shutdown tracer provider to flush remaining spans
    global::shutdown_tracer_provider();

    Ok(())
}
