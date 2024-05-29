use crate::{
    auth::{
        self,
        authorizer::{Authorizer, FileAuthorizer},
    },
    prove, Args,
};
use axum::Router;
use prove::errors::ServerError;
use reqwest::{header, Client};
use serde_json::{Value,json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use std::{net::SocketAddr, time::Duration};
use tokio::net::TcpListener;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utils::shutdown::shutdown_signal;

#[derive(Debug, Clone)]
pub struct AppState {
    pub prover_image_name: String,
    pub message_expiration_time: usize,
    pub session_expiration_time: usize,
    pub jwt_secret_key: String,
    pub nonces: Arc<Mutex<HashMap<String, String>>>,
    pub authorizer: Authorizer,
}

async fn create_account(client: &Client, new_account_url: &str, jose_object: &str) -> Result<Value, reqwest::Error> {
    let res = client.post(new_account_url)
        .header(header::ACCEPT, "application/jose+json")
        .body(body).
        send().await?;
        // .header(header::CONTENT_TYPE, "application/jose+json")
    res.json().await
}


pub async fn start(args: Args) -> Result<(), ServerError> {
    let client = Client::new();
    let directory_url = "https://acme-v02.api.letsencrypt.org/directory";
    let res = client.get(directory_url).send().await.unwrap();
    let directory: Value = res.json().await.unwrap();
    let new_account_url = directory["newAccount"].as_str().unwrap();
    let new_account = create_account(&client, new_account_url,).await.unwrap();
    println!("{:?}", new_account);

    // Enable tracing.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let authorizer = match args.authorized_keys_path {
        Some(path) => {
            tracing::trace!("Using authorized keys file");
            Authorizer::Persistent(FileAuthorizer::new(path).await?)
        }
        None => {
            let authorized_keys = args.authorized_keys.unwrap_or_default();
            tracing::trace!("Using memory authorization");
            Authorizer::Memory(authorized_keys.into())
        }
    };

    let state = AppState {
        prover_image_name: "Sample".to_string(),
        nonces: Arc::new(Mutex::new(HashMap::new())),
        message_expiration_time: args.message_expiration_time as usize,
        session_expiration_time: args.session_expiration_time as usize,
        jwt_secret_key: args.jwt_secret_key,
        authorizer,
    };

    // Create a regular axum app.
    let app = Router::new()
        .nest("/", auth::auth(&state))
        .nest("/prove", prove::router(&state))
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(300)),
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
