#[derive(Debug, Clone)]

pub struct Config {
    pub issuer: IssuerConfig,
    pub certificate: CertificateConfig,
}
#[derive(Debug, Clone)]

pub struct IssuerConfig {
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct CertificateConfig {
    pub  domain: String,
    pub challenge: String,
}