use reqwest::Response;
use reqwest::Client;
use serde_json::Value;
pub async fn post_dns_record(body: String, domain: &str,api_token: &str,zone_id: &str) -> Response {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
        zone_id
    );
    let client = reqwest::Client::new();

    let response = client
        .post(&url)
        .bearer_auth(api_token)
        .body(format!(
            r#"{{
            "type": "TXT",
            "name": "_acme-challenge.{}",
            "content": "{}  ",
            "ttl": 120
        }}"#,
            domain, body
        ))
        .send()
        .await
        .unwrap();
    response
}
pub async fn get_acme_challenge_record_ids(
    api_token: &str,
    zone_id: &str,
    domain: &str,
) -> Option<Vec<String>> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records?type=TXT&name=_acme-challenge.{}",
        zone_id, domain
    );

    let client = Client::new();

    let response = client
        .get(&url)
        .bearer_auth(api_token)
        .send()
        .await
        .unwrap();

    if response.status().is_success() {
        let body = response.text().await.unwrap();
        let json: Value = serde_json::from_str(&body).unwrap();

        let mut ids = Vec::new();

        // Check each record to see if it's a TXT record with the desired prefix
        if let Some(records) = json["result"].as_array() {
            for record in records {
                if record["type"] == "TXT"
                    && record["name"]
                        .as_str()
                        .map_or(false, |n| n.starts_with("_acme-challenge"))
                {
                    if let Some(id) = record["id"].as_str() {
                        ids.push(id.to_string());
                    }
                }
            }
        }
        Some(ids)
    } else {
        None
    }
}
pub async fn delete_dns_record(api_token: &str, zone_id: &str, domain: &str) {
    let ids = get_acme_challenge_record_ids(api_token, zone_id, domain)
        .await
        .unwrap();
    let client = reqwest::Client::new();
    for id in ids {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, id
        );
        let _ = client.delete(&url).bearer_auth(api_token).send().await;
    }
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_post_dns_record() {
        let api_token = "bjlYz_K2uEn278Bcp2GY8hVEgokT-GZsOnFH2otq";
        let zone_id = "c99a975281977d4a887921558d4fd76d";
        let domain = "mateuszchudy.lat";
        let body = "test".to_string();
        let response = super::post_dns_record(body, domain,&api_token,&zone_id).await;
        println!("{:?}", response.text().await.unwrap());
        //assert_eq!(response.status().as_u16(), 200);
    }
    #[tokio::test]
    async fn test_get_dns_record_id() {
        let api_token = "bjlYz_K2uEn278Bcp2GY8hVEgokT-GZsOnFH2otq";
        let zone_id = "c99a975281977d4a887921558d4fd76d";
        let domain = "mateuszchudy.lat";     
        println!(
            "{:?}",
            super::get_acme_challenge_record_ids(api_token, zone_id, domain)
                .await
                .unwrap()
        );
    }
    #[tokio::test]
    async fn test_delete_dns_record() {
        let api_token = "bjlYz_K2uEn278Bcp2GY8hVEgokT-GZsOnFH2otq";
        let zone_id = "c99a975281977d4a887921558d4fd76d";
        let domain = "mateuszchudy.lat";
        super::delete_dns_record(api_token, zone_id, domain).await;
}
}