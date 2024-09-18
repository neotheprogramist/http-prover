use crate::auth::auth;
use crate::auth::auth_errors::AuthorizerError;
use crate::auth::authorizer::{AuthorizationProvider, Authorizer, FileAuthorizer};
use crate::errors::ProverError;
use crate::extractors::workdir::TempDirHandle;
use crate::sse::sse_handler;
use crate::threadpool::ThreadPool;
use crate::utils::job::{get_job, JobStore};
use crate::utils::shutdown::shutdown_signal;
use crate::verifier::verify_proof;
use crate::{prove, Args};
use axum::{
    middleware,
    routing::{get, post},
    serve, Router,
};
use core::net::SocketAddr;
use ed25519_dalek::VerifyingKey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::Mutex;
use tracing::trace;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type NonceString = String;
#[derive(Clone)]
pub struct AppState {
    pub job_store: JobStore,
    pub thread_pool: Arc<Mutex<ThreadPool>>,
    pub message_expiration_time: usize,
    pub session_expiration_time: usize,
    pub jwt_secret_key: String,
    pub nonces: Arc<Mutex<HashMap<NonceString, VerifyingKey>>>,
    pub authorizer: Authorizer,
    pub admin_keys: Vec<VerifyingKey>,
    pub sse_tx: Arc<Mutex<Sender<String>>>,
}

pub async fn start(args: Args) -> Result<(), ProverError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let authorizer =
        Authorizer::Persistent(FileAuthorizer::new(args.authorized_keys_path.clone()).await?);
    let mut admin_keys = Vec::new();
    for key in args.admin_keys {
        let verifying_key_bytes = prefix_hex::decode::<Vec<u8>>(key)
            .map_err(|e| AuthorizerError::PrefixHexConversionError(e.to_string()))?;
        let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes.try_into()?)?;
        admin_keys.push(verifying_key);
        authorizer.authorize(verifying_key).await?;
    }

    for key in args.authorized_keys.iter() {
        let verifying_key_bytes = prefix_hex::decode::<Vec<u8>>(key)
            .map_err(|e| AuthorizerError::PrefixHexConversionError(e.to_string()))?;
        let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes.try_into()?)?;
        authorizer.authorize(verifying_key).await?;
    }
    let (sse_tx, _) = broadcast::channel(100);
    let app_state = AppState {
        message_expiration_time: args.message_expiration_time,
        session_expiration_time: args.session_expiration_time,
        jwt_secret_key: args.jwt_secret_key,
        nonces: Arc::new(Mutex::new(HashMap::new())),
        authorizer,
        job_store: JobStore::default(),
        thread_pool: Arc::new(Mutex::new(ThreadPool::new(args.num_workers))),
        admin_keys,
        sse_tx: Arc::new(Mutex::new(sse_tx)),
    };

    async fn ok_handler() -> &'static str {
        "OK"
    }

    let app = Router::new()
        .route("/", get(ok_handler))
        .route("/verify", post(verify_proof))
        .route("/get-job/:id", get(get_job))
        .route("/sse", get(sse_handler))
        .with_state(app_state.clone())
        .nest("/", auth(app_state.clone()))
        .nest("/prove", prove::router(app_state.clone()))
        .layer(middleware::from_extractor::<TempDirHandle>());

    let address: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .map_err(ProverError::AddressParse)?;

    let listener = TcpListener::bind(address).await?;

    trace!("Listening on {}", address);

    let keys = args.authorized_keys.clone();
    trace!("provided public keys {:?}", keys);

    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(app_state.thread_pool))
        .await?;

    Ok(())
}
