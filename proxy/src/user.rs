use crate::config::{Config, ForwardConfig, get_config};
use chrono::{DateTime, Utc};
use common::user::repo::{FileSystemUserRepository, UserActiveProxyAddressRepo};
use common::user::{User, UserRepository, UserWithExpiredTime, UserWithProxyServers};
use crypto::RsaCrypto;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::OnceLock;
static USER_REPO: OnceLock<FileSystemUserRepository<ProxyUser, Config>> = OnceLock::new();
static FORWARD_USER_REPO: OnceLock<Option<FileSystemUserRepository<ForwardUser, ForwardConfig>>> =
    OnceLock::new();

/// Get the repository of the proxy user.
pub fn get_user_repo() -> &'static FileSystemUserRepository<ProxyUser, Config> {
    USER_REPO.get_or_init(|| {
        FileSystemUserRepository::<ProxyUser, Config>::new(get_config())
            .expect("Fail to create user repository from file system")
    })
}

/// Get the repository of the forwarding user.
pub fn get_forward_user_repo()
-> Option<&'static FileSystemUserRepository<ForwardUser, ForwardConfig>> {
    FORWARD_USER_REPO
        .get_or_init(|| {
            let forward_user_repo = FileSystemUserRepository::<ForwardUser, ForwardConfig>::new(
                get_config().forward()?,
            )
            .ok()?;
            Some(forward_user_repo)
        })
        .as_ref()
}

/// The user in proxy side
#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyUser {
    username: String,
    expired_time: Option<DateTime<Utc>>,
    #[serde(skip)]
    rsa_crypto: Option<RsaCrypto>,
}

impl User for ProxyUser {
    fn username(&self) -> &str {
        &self.username
    }
    fn rsa_crypto(&self) -> Option<&RsaCrypto> {
        self.rsa_crypto.as_ref()
    }
    fn set_rsa_crypto(&mut self, rsa_crypto: RsaCrypto) {
        self.rsa_crypto = Some(rsa_crypto)
    }
}
impl UserWithExpiredTime for ProxyUser {
    fn expired_time(&self) -> Option<&DateTime<Utc>> {
        self.expired_time.as_ref()
    }
}

/// The user for forwarding connection
#[derive(Serialize, Deserialize, Debug)]
pub struct ForwardUserConfig {
    username: String,
    proxy_servers: Vec<SocketAddr>,
}

pub struct ForwardUser {
    config: ForwardUserConfig,
    rsa_crypto: Option<RsaCrypto>,
    user_active_proxy_address_repo: UserActiveProxyAddressRepo,
}

impl User for ForwardUser {
    fn username(&self) -> &str {
        &self.config.username
    }
    fn rsa_crypto(&self) -> Option<&RsaCrypto> {
        self.rsa_crypto.as_ref()
    }
    fn set_rsa_crypto(&mut self, rsa_crypto: RsaCrypto) {
        self.rsa_crypto = Some(rsa_crypto)
    }
}

impl UserWithProxyServers for ForwardUser {
    fn proxy_servers(&self) -> &[SocketAddr] {
        &self.config.proxy_servers
    }
    fn set_active_proxy_addr_repo(
        &mut self,
        user_active_proxy_address_repo: UserActiveProxyAddressRepo,
    ) {
        self.user_active_proxy_address_repo = user_active_proxy_address_repo;
    }
}
