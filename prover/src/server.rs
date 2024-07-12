use crate::{
    auth::{
        self,
        authorizer::{Authorizer, FileAuthorizer},
    },
    prove, AcmeArgs, Args,
};
use axum::Router;
use lib_acme::cert::cert_manager::{issue_certificate, read_cert, renew_certificate};
use prove::errors::ServerError;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use std::{net::SocketAddr, time::Duration};
use tokio::net::TcpListener;
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
            RequestBodyLimitLayer::new(100 * 1024 * 1024),
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(60 * 60)),
        ));

    let address: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    tracing::trace!("start listening on {}", address);

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind(address).await?;

    tokio::spawn(async move {
        let _result = async {
            let challange_type = lib_acme::cert::types::ChallangeType::Dns01;

            issue_certificate(
                acme_args.contact_mails.clone(),
                acme_args.domain_identifiers(),
                challange_type.clone(),
                acme_args.api_token.as_str(),
                acme_args.zone_id.as_str(),
                &acme_args.url,
                &acme_args.cert_path,
            )
            .await?;

            let cert = read_cert(&acme_args.cert_path)?;

            renew_certificate(
                acme_args.contact_mails.clone(),
                acme_args.domain_identifiers(),
                challange_type,
                &acme_args.api_token,
                &acme_args.zone_id,
                &acme_args.url,
                &cert,
                &acme_args.cert_path,
                acme_args.renewal_threshold,
            )
            .await?;

            Ok::<(), lib_acme::cert::errors::AcmeErrors>(())
        }
        .await;
    });

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}
