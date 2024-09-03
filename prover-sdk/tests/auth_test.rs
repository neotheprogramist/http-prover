#[cfg(test)]
mod tests {
    use prover_sdk::{access_key::ProverAccessKey, sdk::ProverSDK};
    use url::Url;

    #[tokio::test]
    async fn test_authorized_access() {
        let private_key = std::env::var("PRIVATE_KEY").unwrap();
        let url = std::env::var("PROVER_URL").unwrap();
        let access_key = ProverAccessKey::from_hex_string(&private_key).unwrap();
        let url = Url::parse(&url).unwrap();
        let _sdk = ProverSDK::new(url, access_key).await.unwrap();
    }
    #[tokio::test]
    async fn test_unauthorized_access() {
        let unauthorized_key = ProverAccessKey::generate();
        let url = std::env::var("PROVER_URL").unwrap();
        let url = Url::parse(&url).unwrap();
        let sdk = ProverSDK::new(url, unauthorized_key).await;
        assert!(sdk.is_err());
    }
}
