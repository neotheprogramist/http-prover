use crate::{
    auth::{
        self,
        authorizer::{Authorizer, FileAuthorizer},
    },
    prove::{self},
    AcmeArgs, Args,
};
use axum::Router;
use axum_server::tls_openssl::OpenSSLConfig;
use lib_acme::cert::{cert_manager::CertificateManager, errors::AcmeErrors, types::ChallangeType};
use prove::errors::ServerError;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use std::{net::SocketAddr, time::Duration};
use tokio::{select, sync::oneshot};
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer, trace::TraceLayer};
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

pub async fn start(args: Args, acme_args: AcmeArgs) -> Result<(), ServerError> {
    // Enable tracing for logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize the CertificateManager for handling SSL certificates
    let cert_manager = CertificateManager::new(
        acme_args.contact_mails,
        acme_args.domain_identifiers,
        ChallangeType::Dns01,
        acme_args.api_token,
        acme_args.zone_id,
        acme_args.url,
        acme_args.renewal_threshold,
    );
    cert_manager.issue_certificate().await?;

    // Get the certificate and key in PEM format
    let cert = cert_manager
        .get_cert()
        .await?
        .ok_or(AcmeErrors::MutexPoisonedError(
            "Failed to acquire cert lock".to_string(),
        ))?
        .to_pem()
        .map_err(|_| AcmeErrors::ConversionError)?;
    let key = cert_manager
        .get_key_pem()
        .await?
        .ok_or(AcmeErrors::ConversionError)?;

    // Create OpenSSL configuration from the PEM certificate and key
    let config = OpenSSLConfig::from_pem(&cert, &key)
        .map_err(|e| ServerError::ConfigCreateError(e.to_string()))?;

    // Initialize the authorizer based on provided arguments
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

    // Create application state
    let state = AppState {
        prover_image_name: "Sample".to_string(),
        nonces: Arc::new(Mutex::new(HashMap::new())),
        message_expiration_time: args.message_expiration_time as usize,
        session_expiration_time: args.session_expiration_time as usize,
        jwt_secret_key: args.jwt_secret_key,
        authorizer,
    };

    let handle = axum_server::Handle::new();
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

    // Create the axum application with routes and middleware
    let app = Router::new()
        .nest("/", auth::auth(&state))
        .nest("/prove", prove::router(&state))
        .layer((
            RequestBodyLimitLayer::new(100 * 1024 * 1024), // Limit request body size to 100MB
            TraceLayer::new_for_http(), // Enable HTTP tracing
            TimeoutLayer::new(Duration::from_secs(60 * 60)), // Set request timeout to 1 hour
        ));

    // Parse the socket address from arguments
    let address: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    tracing::trace!("start listening on {}", address.clone());

    let config_clone = config.clone();
    let _renew_handle: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        // Task to renew the SSL certificate periodically
        let _result = async move {
            loop {
                cert_manager.renew_certificate().await?;
                let new_cert = cert_manager
                    .get_cert()
                    .await?
                    .ok_or(AcmeErrors::MutexPoisonedError(
                        "Failed to acquire cert lock".to_string(),
                    ))?
                    .to_pem()
                    .map_err(|_| AcmeErrors::ConversionError)?;
                let new_key = cert_manager
                    .get_key_pem()
                    .await?
                    .ok_or(AcmeErrors::ConversionError)?;
                config_clone
                    .reload_from_pem(&new_cert, &new_key)
                    .map_err(|e| ServerError::ConfigReloadError(e.to_string()))?;
                select! {
                    _ = &mut shutdown_rx => {
                        tracing::info!("renewal task received shutdown signal");
                        break;
                    }
                }
            }
            Ok::<(), ServerError>(())
        }
        .await;
    });

    // Start the server with OpenSSL configuration
    let server = axum_server::bind_openssl(address, config.clone())
        .handle(handle.clone())
        .serve(app.clone().into_make_service());

    // Wait for either the server to exit or a shutdown signal
    tokio::select! {
        result = server => {
            if let Err(err) = result {
                tracing::error!("server error: {}", err);
            }
        },
        _ = shutdown_signal(handle.clone()) => {
            tracing::info!("shutdown signal received");
            let _ = shutdown_tx.send(());
        }
    }
    Ok(())
}
