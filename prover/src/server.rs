use crate::{
    auth::{
        self,
        authorizer::{Authorizer, FileAuthorizer},
    }, https::models::{CertificateConfig, Config, IssuerConfig}, prove, Args
};
use axum::Router;
use prove::errors::ServerError;
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
    pub tokens: Arc<Mutex<HashMap<String, String>>>, // For ACME challenge tokens
    pub authorizer: Authorizer,
    pub config: Config,
}

pub async fn start(args: Args) -> Result<(), ServerError> {
    // Enable tracing.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();


    //sample config file TODO: change later
    let config = Config {
        issuer: IssuerConfig {
            email: "your-email@example.com".to_string(),
        },
        certificate: CertificateConfig {
            domain: "example.com".to_string(),
            challenge: "http-01".to_string(),
        },
    };
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


    let state = Arc::new(AppState {
        prover_image_name: "Sample".to_string(),
        nonces: Arc::new(Mutex::new(HashMap::new())),
        tokens: Arc::new(Mutex::new(HashMap::new())), // Initialize tokens map
        message_expiration_time: args.message_expiration_time as usize,
        session_expiration_time: args.session_expiration_time as usize,
        jwt_secret_key: args.jwt_secret_key,
        authorizer,
        config,
    });

    // Obtain the initial certificate
    state.obtain_certificate().await?;

    // Spawn the renewal task
    let app_state_clone = state.clone();
    tokio::spawn(async move {
        app_state_clone.renewal_task().await;
    });
    // Create a regular axum app.
    let app = Router::new()
        .nest("/", auth::auth(&state))
        .nest("/prove", prove::router(&state))
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(300)),
        ));

    let cert_file = &mut BufReader::new(File::open("fullchain.pem")?);
    let key_file = &mut BufReader::new(File::open("key.pem")?);

    let cert_chain = certs(cert_file)?.into_iter().map(Certificate).collect();
    let mut keys = pkcs8_private_keys(key_file)?;
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, PrivateKey(keys.remove(0)))?;

    let tls_acceptor = TlsAcceptor::from(Arc::new(config));

    let address: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    tracing::trace!("start listening on {}", address);

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind(address).await?;
    let incoming = tokio::net::TcpListener::from_std(listener.into_std()?)?;

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())

    
}
