use crate::config::{Config, get_config};
use common::user::repo::FileSystemUserRepository;
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
pub struct AgentUser {
    proxy_servers: Vec<SocketAddr>,
    username: String,
    #[serde(skip)]
    rsa_crypto: Option<RsaCrypto>,
}
impl UserWithProxyServers for AgentUser {
    fn proxy_servers(&self) -> &[SocketAddr] {
        &self.proxy_servers
    }
}
impl User for AgentUser {
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
