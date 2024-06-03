use crate::{
    auth::{
        self,
        authorizer::{Authorizer, FileAuthorizer},
    }, cert::{cert_menager::{fetch_authorizations, fetch_challanges, get_challanges_tokens, new_directory, new_nonce, respond_to_challange, submit_order}, create_jws::create_jws}, prove, Args
};
use axum::{extract::Path, response::IntoResponse, Extension, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use josekit::{jwk::alg::ec::{EcCurve, EcKeyPair}, jwt::JwtPayload};
use prove::errors::ServerError;
use reqwest::{get, header, Client, StatusCode};
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
use openssl::{hash::{hash, MessageDigest}, pkey::PKey};

#[derive(Debug, Clone)]
pub struct AppState {
    pub prover_image_name: String,
    pub message_expiration_time: usize,
    pub session_expiration_time: usize,
    pub token : String,
    pub thumbprint: String,
    pub jwt_secret_key: String,
    pub nonces: Arc<Mutex<HashMap<String, String>>>,
    pub authorizer: Authorizer,
}
async fn place_token(Path(token): Path<String>, Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let token = state.token.clone();
    let thumbprint = state.thumbprint.clone();
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .body(format!("{}.{}",token.clone(),thumbprint.clone()))
        .unwrap()
}


pub async fn start(args: Args) -> Result<(), ServerError> {
    let ec_key_pair = EcKeyPair::generate(EcCurve::P256).unwrap();
    let client = Client::new();
    let urls = new_directory().await;

    let account = crate::cert::cert_menager::new_account(&client, urls.clone(),"mateusz@gmail.com".to_string(),ec_key_pair.clone()).await;
    let account_url = account.headers().get("location").unwrap().to_str().unwrap();

    let order = submit_order(&client, urls,vec!["prover.visoft.dev".to_string()],ec_key_pair.clone(),account_url.to_string()).await;
    let authorizations = fetch_authorizations(order).await;
    let challanges = fetch_challanges(authorizations).await;
    let tokens = get_challanges_tokens(challanges.clone()).await;
    let chall_responese = respond_to_challange(challanges[0].clone(), ec_key_pair.clone(), account_url.to_string()).await;
    println!("{:?}",chall_responese);
    let public_key = ec_key_pair.to_der_public_key().clone();
    let digest = hash(MessageDigest::sha256(), &public_key).unwrap();
    let thumbprint = URL_SAFE_NO_PAD.encode(digest);
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
        token: tokens[0].clone(),
        thumbprint: thumbprint.clone(),
        jwt_secret_key: args.jwt_secret_key,
        authorizer,
    };

    // Create a regular axum app.
    let app = Router::new()
        .nest("/", auth::auth(&state))
        .nest("/prove", prove::router(&state))
        .route("/.well-known/acme-challenge/:token",axum::routing::method_routing::get(place_token))
        .layer(Extension(Arc::new(state)))
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
