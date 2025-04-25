use chrono::{DateTime, Utc};
use ppaass_2025_crypto::RsaCrypto;
use ppaass_2025_user::UserInfo;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct ProxyUserInfo {
    username: String,
    expired_time: Option<DateTime<Utc>>,
    #[serde(skip)]
    rsa_crypto: Option<RsaCrypto>,
}
impl UserInfo for ProxyUserInfo {
    fn username(&self) -> &str {
        &self.username
    }
    fn expired_time(&self) -> Option<&DateTime<Utc>> {
        self.expired_time.as_ref()
    }
    fn rsa_crypto(&self) -> Option<&RsaCrypto> {
        self.rsa_crypto.as_ref()
    }
    fn attach_rsa_crypto(&mut self, rsa_crypto: RsaCrypto) {
        self.rsa_crypto = Some(rsa_crypto)
    }
}
