use axum::{routing::get, Router};
use std::{
    net::SocketAddr,
    time::Duration,
};
use tokio::{net::TcpListener,time::sleep};
use tower_http::{timeout::TimeoutLayer,trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utils::shutdown::shutdown_signal;
use crate::{prove, Args};
use std::{collections::HashMap,sync::{Arc, Mutex}};
use prove::errors::ServerError;

#[derive(Debug, Clone)]
pub struct AppState {
    pub prover_image_name: String,
    pub nonces: Arc<Mutex<HashMap<String, String>>>,
}

pub async fn start(args: &Args) -> Result<(), ServerError> {
    let state: AppState = AppState {
        prover_image_name: "Sample".to_string(),
        nonces: Arc::new(Mutex::new(HashMap::new())),
    };
    // Enable tracing.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create a regular axum app.
    let app = Router::new()
        .nest("/", prove::auth(&state))
        .nest("/prove", prove::router())
        .route("/slow", get(|| sleep(Duration::from_secs(5))))
        .route("/forever", get(std::future::pending::<()>))
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(60)),
        ));

    let address: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    tracing::trace!("start listening on {}", address);

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind(address).await?;

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
