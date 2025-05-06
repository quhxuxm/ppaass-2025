use chrono::{DateTime, Utc};
use ppaass_2025_crypto::RsaCrypto;
pub trait BasicUser {
    /// The username
    fn username(&self) -> &str;
    /// The expired time
    fn expired_time(&self) -> Option<&DateTime<Utc>>;
    /// The rsa crypto
    fn rsa_crypto(&self) -> Option<&RsaCrypto>;
    fn attach_rsa_crypto(&mut self, rsa_crypto: RsaCrypto);
}
