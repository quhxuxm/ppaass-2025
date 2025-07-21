use crate::config::{Config, get_config};
use common::user::repo::{FileSystemUserRepository, UserActiveProxyAddressRepo};
use common::user::{User, UserRepository, UserWithProxyServers};
use crypto::RsaCrypto;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::OnceLock;
pub static AGENT_USER_REPO: OnceLock<FileSystemUserRepository<AgentUser, Config>> = OnceLock::new();
pub fn get_agent_user_repo() -> &'static FileSystemUserRepository<AgentUser, Config> {
    AGENT_USER_REPO.get_or_init(|| {
        FileSystemUserRepository::<AgentUser, Config>::new(get_config())
            .expect("Fail to create user repository from file system")
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentUserConfig {
    proxy_servers: Vec<SocketAddr>,
    username: String,
}

pub struct AgentUser {
    config: AgentUserConfig,
    rsa_crypto: Option<RsaCrypto>,
    user_active_proxy_address_repo: UserActiveProxyAddressRepo,
}
impl UserWithProxyServers for AgentUser {
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
impl User for AgentUser {
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
