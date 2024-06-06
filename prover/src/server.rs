use crate::{
    auth::{
        self,
        authorizer::{Authorizer, FileAuthorizer},
    },
    cert::cert_menager::{fetch_authorizations, fetch_challanges, fetch_order_status, generate_csr, get_challanges_tokens, get_key_authorization, new_directory, order_finalization, post_dns_record, respond_to_challange, submit_order}, prove, Args};
use axum::{http::response, Router};
use josekit::jwk::alg::ec::{EcCurve, EcKeyPair};
use prove::errors::ServerError;
use reqwest::Client;
use serde_json::Value;
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

pub async fn start(args: Args) -> Result<(), ServerError> {
    let ec_key_pair = EcKeyPair::generate(EcCurve::P256).unwrap();
    let client = Client::new();
    let urls = new_directory().await;

    let account = crate::cert::cert_menager::new_account(
        &client,
        urls.clone(),
        "mateusz@gmail.com".to_string(),
        ec_key_pair.clone(),
    )
    .await;
   
    let account_url = account
    .headers()
    .get("location")
    .ok_or("Location header missing").unwrap()
    .to_str().unwrap();

    let order = submit_order(
        &client,
        urls.clone(),
        vec!["mateuszchudy.lat".to_string()],
        ec_key_pair.clone(),
        account_url.to_string(),
    )
    .await;

    let order_url = order.headers()
        .get("location")
        .ok_or("Location header missing").unwrap()
        .to_str().unwrap()
        .to_owned();  // Make an owned copy of the URL

    println!("Order URL: {}", order_url);
    // Deserialize the JSON body for further processing
    let order_body: Value = order.json().await.unwrap();

    // Now that we have both `order_url` and `order_body`, we no longer need the original `order`
    let authorizations = fetch_authorizations(order_body).await;
    let challenges = fetch_challanges(authorizations).await;

    let tokens = get_challanges_tokens(challenges.clone()).await;
    let encoded_digest = get_key_authorization(tokens[0].clone(), ec_key_pair.clone());

    // Post the DNS record
    post_dns_record(encoded_digest.clone()).await;
    println!("DNS record posted, waiting for the DNS changes to propagate...");
    // Wait for the DNS changes to propagate
    for i in 1..13 {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        println!("Waiting for DNS changes to propagate... time passed :{} seconds", i*10);
    }
    println!("DNS changes should have propagated by now.");
    // Respond to the challenge
    respond_to_challange(challenges[0].clone(), ec_key_pair.clone(), account_url.to_string().clone()).await;

    println!("Challenge responded to, waiting for the order to complete...");
    loop {
        let order_status = fetch_order_status(&client, &order_url).await.unwrap();
        let status = order_status["status"].as_str().unwrap_or("unknown");

        match status {
            "valid" => {
                println!("Order is completed successfully. Downloading certificate...");
                let certificate_url = order_status["certificate"].as_str().unwrap();
                let certificate = client.get(certificate_url).send().await.unwrap();
                let certificate_body = certificate.text().await.unwrap();
                println!("Certificate: \n {}", certificate_body);
                break;
            },
            "invalid" => {
                println!("Order has failed.");
                break;
            },
            "pending"  => {
                println!("Order is pending...");
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            },
            "ready" => {
                println!("Order is ready... finalizing.");
                let finalization_url = order_status["finalize"].as_str().unwrap();
                let csr = generate_csr("mateuszchudy.lat").unwrap();
                order_finalization(csr, urls.clone(), ec_key_pair.clone(), account_url.to_string(), finalization_url.to_string()).await;
            },
            "processing" => {
                println!("Order is processing...");
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            },
            _ => {
                println!("Order status: {}", status);
                break;
            }
        }
    }
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
