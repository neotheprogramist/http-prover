use std::path::PathBuf;

use prover::server::start;
use prover::{AcmeArgs, Args};
use prover_sdk::ProverAccessKey;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use url::Url;

pub async fn spawn_prover() -> (JoinHandle<()>, ProverAccessKey, Url) {
    use url::Url;

    let port = TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap()
        .local_addr()
        .unwrap()
        .port();

    let key = ProverAccessKey::generate();
    let encoded_key = prefix_hex::encode(key.0.verifying_key().to_bytes());

    let args = Args {
        host: "0.0.0.0".to_string(),
        port,
        jwt_secret_key: "placeholder".to_string(),
        message_expiration_time: 60,
        session_expiration_time: 3600,
        authorized_keys: Some(vec![encoded_key]),
        authorized_keys_path: None,
    };
    let acme_args = AcmeArgs::default();
    
    let handle = tokio::spawn(async move {
        start(args,acme_args).await.unwrap();
    });

    let url = Url::parse(&format!("http://localhost:{}", port)).unwrap();

    (handle, key, url)
}

pub async fn read_file(path: PathBuf) -> Result<String, std::io::Error> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file_path = manifest_dir.join("../../").join(path);
    println!("Reading file: {:?}", file_path);

    let mut file = File::open(file_path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}
