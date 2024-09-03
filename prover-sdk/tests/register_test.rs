use prover_sdk::{access_key::ProverAccessKey, sdk::ProverSDK};
use url::Url;

#[tokio::test]
async fn test_register_authorized() {
    let url = std::env::var("PROVER_URL").unwrap();
    let admin_key = std::env::var("ADMIN_PRIVATE_KEY").unwrap();
    let admin_key = ProverAccessKey::from_hex_string(&admin_key).unwrap();
    let random_key = ProverAccessKey::generate();
    let url = Url::parse(&url).unwrap();
    let mut sdk = ProverSDK::new(url.clone(), admin_key).await.unwrap();
    sdk.register(random_key.0.verifying_key()).await.unwrap();
    let new_sdk = ProverSDK::new(url, random_key).await;
    assert!(new_sdk.is_ok());
}
#[tokio::test]
async fn test_register_unauthorized() {
    let url = std::env::var("PROVER_URL").unwrap();
    let authorized_key = std::env::var("PRIVATE_KEY").unwrap();
    let authorized_key = ProverAccessKey::from_hex_string(&authorized_key).unwrap();
    let url = Url::parse(&url).unwrap();
    //Connecting with sdk using authorized key, but not recognized as admin key
    let mut sdk = ProverSDK::new(url.clone(), authorized_key).await.unwrap();
    let random_key = ProverAccessKey::generate();
    //Trying to register a new key using not admin key, should return error
    let response = sdk.register(random_key.0.verifying_key()).await;
    assert!(response.is_err());
    let new_sdk = ProverSDK::new(url, random_key).await;
    assert!(new_sdk.is_err());
}
