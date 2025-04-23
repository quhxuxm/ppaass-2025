use chrono::{DateTime, Utc};
use ppaass_2025_user::UserInfo;
pub struct ProxyUserInfo {
    username: String,
    expired_time: Option<DateTime<Utc>>,
}
impl UserInfo for ProxyUserInfo {
    fn username(&self) -> &str {
        &self.username
    }
    fn expired_time(&self) -> Option<&DateTime<Utc>> {
        self.expired_time.as_ref()
    }
}
