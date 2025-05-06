use chrono::{DateTime, Utc};
use ppaass_2025_common::user::{BasicUser, ExpiredUser};
use ppaass_2025_crypto::RsaCrypto;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyUserInfo {
    username: String,
    expired_time: Option<DateTime<Utc>>,
    #[serde(skip)]
    rsa_crypto: Option<RsaCrypto>,
}

impl BasicUser for ProxyUserInfo {
    fn username(&self) -> &str {
        &self.username
    }
    fn rsa_crypto(&self) -> Option<&RsaCrypto> {
        self.rsa_crypto.as_ref()
    }
    fn attach_rsa_crypto(&mut self, rsa_crypto: RsaCrypto) {
        self.rsa_crypto = Some(rsa_crypto)
    }
}
impl ExpiredUser for ProxyUserInfo {
    fn expired_time(&self) -> Option<&DateTime<Utc>> {
        self.expired_time.as_ref()
    }
}
