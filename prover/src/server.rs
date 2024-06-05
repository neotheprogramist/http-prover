use crate::{
    auth::{
        self,
        authorizer::{Authorizer, FileAuthorizer},
    }, cert::{cert_menager::{fetch_authorizations, fetch_challanges, get_challanges_tokens, new_directory, new_nonce, post_dns_record, respond_to_challange, submit_order}, create_jws::create_jws}, prove, Args
};
use axum::{extract::Path, middleware::future::FromExtractorResponseFuture, response::IntoResponse, Extension, Router};
use base64::{Engine};
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use curve25519_dalek::digest::consts::U64;
use josekit::{jwk::{self, alg::ec::{EcCurve, EcKeyPair}}, jwt::JwtPayload};
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
use openssl::{hash::{hash, MessageDigest}, pkey::{PKey, Public}};

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

pub async fn start(args: Args) -> Result<(), ServerError> {
    let ec_key_pair = EcKeyPair::generate(EcCurve::P256).unwrap();
    let client = Client::new();
    let urls = new_directory().await;

    let account = crate::cert::cert_menager::new_account(&client, urls.clone(),"mateusz@gmail.com".to_string(),ec_key_pair.clone()).await;
    let account_url = account.headers().get("location").unwrap().to_str().unwrap();

    let order = submit_order(&client, urls,vec!["mateuszchudy.lat".to_string()],ec_key_pair.clone(),account_url.to_string()).await;
    let authorizations = fetch_authorizations(order).await;
    let challanges = fetch_challanges(authorizations).await;
    println!("{:?}",challanges.clone());
    let tokens = get_challanges_tokens(challanges.clone()).await;
    println!("{:?}",tokens.clone());

    let jwk_json = ec_key_pair.to_jwk_public_key().clone();
    // Serialize JWK to string (assuming it's already JSON)
    let jwk_string = serde_json::to_string(&jwk_json).unwrap();
    let jwk_string = jwk_string.replace("\n", "").replace(" ", "");

    // Compute SHA-256 hash of the JWK string
    // let jwk_digest = hash(MessageDigest::sha256(), jwk_string.as_bytes()).unwrap();
    let thumbprint = BASE64_URL_SAFE_NO_PAD.encode(&jwk_string.as_bytes());
    println!("{:?}",thumbprint.clone());
    // Construct key authorization using the token and the thumbprint
    let key_authorization = format!("{}.{}", tokens[0], thumbprint);
    println!("{:?}",key_authorization.clone());

    // Compute SHA-256 hash of the key authorization
    let key_auth_digest = hash(MessageDigest::sha256(), key_authorization.as_bytes()).unwrap();
    let encoded_digest = BASE64_URL_SAFE_NO_PAD.encode(&key_auth_digest);
    // Post the DNS record
    println!("{}",encoded_digest.clone());
    let dns_record = post_dns_record(encoded_digest.clone()).await;

    tokio::time::sleep(tokio::time::Duration::from_secs(125)).await; // Wait for initial propagation
    
    let response = respond_to_challange(challanges[0].clone(),ec_key_pair,account_url.to_string()).await;
    println!("{:?}",response);
    let response = client.get(challanges[0].clone()).send().await.unwrap();
    let response_body = response.text().await.unwrap();
    println!("{:?}",response_body);


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
