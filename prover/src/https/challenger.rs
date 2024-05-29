use std::{sync::Arc, time::{Duration, SystemTime}};
use tokio::time::interval;
use crate::server::AppState;
use instant_acme::{Account, ChallengeType, LetsEncrypt, NewOrder, Order};
use rcgen::{generate_simple_self_signed, KeyPair, PKCS_ED25519};



// ACME Account Creation: Creates a new ACME account with Let's Encrypt.
// Order Creation: Creates a new order for the specified domain.
// Challenge Handling: Finds and sets up the appropriate challenge (HTTP-01 or DNS-01) based on the configuration.
// CSR Generation: Uses rcgen to generate a CSR without self-signing it. The CSR is sent to Let's Encrypt for signing.
// Order Finalization: Finalizes the order with the CSR and retrieves the signed certificate.
// Certificate and Private Key Storage: Saves the obtained certificate and private key to files.
impl AppState {
    pub async fn obtain_certificate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Create an ACME account
        let new_account = instant_acme::NewAccount {
            contact: &[&format!("mailto:{}", self.config.issuer.email)],
            terms_of_service_agreed: true,
            only_return_existing: false,
        };
        let account = Account::create(
            &new_account,
            LetsEncrypt::Production.url(),
            None,
        ).await?;

        
        // Create a new order
        let new_order = NewOrder {
            identifiers: &[instant_acme::Identifier::Dns(self.config.certificate.domain.clone())],
        };
        let order = account.0.new_order(&new_order).await?;
        
        // Get authorization and challenge
        let auth = order.authorizations().await?.first().ok_or("No authorization found")?;

        let challenge = auth.challenges.iter()
        .find(|c| match self.config.certificate.challenge.as_str() {
            "http-01" => c.r#type == ChallengeType::Http01,
            "dns-01" => c.r#type == ChallengeType::Dns01,
            _ => false,
        })
        .ok_or("Unsupported challenge type")?;

        {
            let credentials = account.1.key();
            let token = challenge.token;
            // Construct key authorization
            let account_key = account.1;
            let account_key_thumbprint = base64::encode_config(
                Sha256::digest(
                    &serde_json::to_vec(account_key).unwrap()
                ), 
                URL_SAFE_NO_PAD
            );
            let key_authorization = format!("{}.{}", token, account_key_thumbprint);
            let mut tokens = self.tokens.lock().unwrap();
            tokens.insert(token.clone(), key_authorization);
        }

        // Validate the challenge
        challenge.validate().await?;

        // Generate a new private key and CSR
        let private_key = KeyPair::generate()?;
        let csr = generate_simple_self_signed([self.config.certificate.domain.clone()]).unwrap().

        // Finalize the order and obtain the certificate
        let cert = order.finalize(&csr).await?;

        // Save the certificate and private key
        std::fs::write("fullchain.pem", cert.certificate().as_bytes())?;
        std::fs::write("key.pem", private_key.serialize_pem())?;

        Ok(())
    }

    pub async fn renew_certificate(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cert_path = "fullchain.pem";
        let cert_metadata = std::fs::metadata(cert_path)?;
        let modified = cert_metadata.modified()?;
        let expiry_time = modified + Duration::from_secs(90 * 24 * 60 * 60); // 90 days

        let now = SystemTime::now();
        if now >= expiry_time {
            self.obtain_certificate().await?;
        }
        Ok(())
    }

    pub async fn renewal_task(self: Arc<Self>) {
        let mut interval = interval(Duration::from_secs(24 * 60 * 60)); // Check daily
        loop {
            interval.tick().await;
            if let Err(e) = self.renew_certificate().await {
                eprintln!("Failed to renew certificate: {:?}", e);
            }
        }
    }
}

