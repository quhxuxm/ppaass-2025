use crate::error::Error;
pub use rsa::pkcs8::EncodePrivateKey;
pub use rsa::pkcs8::EncodePublicKey;
pub use rsa::pkcs8::LineEnding;
pub use rsa::rand_core::OsRng;
use rsa::{
    Pkcs1v15Encrypt,
    pkcs8::{DecodePrivateKey, DecodePublicKey},
};
pub use rsa::{RsaPrivateKey, RsaPublicKey};
use std::fmt::Debug;
use std::io::Read;
pub const DEFAULT_AGENT_PRIVATE_KEY_PATH: &str = "AgentPrivateKey.pem";
pub const DEFAULT_AGENT_PUBLIC_KEY_PATH: &str = "AgentPublicKey.pem";
pub const DEFAULT_PROXY_PRIVATE_KEY_PATH: &str = "ProxyPrivateKey.pem";
pub const DEFAULT_PROXY_PUBLIC_KEY_PATH: &str = "ProxyPublicKey.pem";
/// The util to do RSA encryption and decryption.
#[derive(Debug)]
pub struct RsaCrypto {
    /// The private used to do decryption
    private_key: RsaPrivateKey,
    /// The public used to do encryption
    public_key: RsaPublicKey,
}
impl RsaCrypto {
    pub fn new<A, B>(mut public_key_read: A, mut private_key_read: B) -> Result<Self, Error>
    where
        A: Read + Debug,
        B: Read + Debug,
    {
        let mut public_key_string = String::new();
        public_key_read.read_to_string(&mut public_key_string)?;
        let public_key = RsaPublicKey::from_public_key_pem(&public_key_string)?;
        let mut private_key_string = String::new();
        private_key_read.read_to_string(&mut private_key_string)?;
        let private_key = RsaPrivateKey::from_pkcs8_pem(&private_key_string)?;
        Ok(Self {
            public_key,
            private_key,
        })
    }
    /// Encrypt the target bytes with RSA public key
    pub fn encrypt(&self, target: &[u8]) -> Result<Vec<u8>, Error> {
        let result = self
            .public_key
            .encrypt(&mut OsRng, Pkcs1v15Encrypt, target.as_ref())?;
        Ok(result)
    }
    /// Decrypt the target bytes with RSA private key
    pub fn decrypt(&self, target: &[u8]) -> Result<Vec<u8>, Error> {
        let result = self.private_key.decrypt(Pkcs1v15Encrypt, target.as_ref())?;
        Ok(result)
    }
}
