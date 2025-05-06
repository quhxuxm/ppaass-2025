use chrono::{DateTime, Utc};
use ppaass_2025_common::user::user::BasicUser;
use ppaass_2025_crypto::RsaCrypto;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
#[derive(Serialize, Deserialize, Debug)]
pub struct AgentUserInfo {
    proxy_servers: Vec<SocketAddr>,
    username: String,
    #[serde(skip)]
    rsa_crypto: Option<RsaCrypto>,
}
impl AgentUserInfo {
    pub fn proxy_servers(&self) -> &[SocketAddr] {
        &self.proxy_servers
    }
}
impl BasicUser for AgentUserInfo {
    fn username(&self) -> &str {
        &self.username
    }
    fn expired_time(&self) -> Option<&DateTime<Utc>> {
        None
    }
    fn rsa_crypto(&self) -> Option<&RsaCrypto> {
        self.rsa_crypto.as_ref()
    }
    fn attach_rsa_crypto(&mut self, rsa_crypto: RsaCrypto) {
        self.rsa_crypto = Some(rsa_crypto)
    }
}
