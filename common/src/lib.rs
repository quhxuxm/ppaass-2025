mod server;
mod config;
mod error;
mod runtime;
mod log;
mod codec;
pub use codec::SecureLengthDelimitedCodec;
pub use config::CoreLogConfig;
pub use config::CoreRuntimeConfig;
pub use config::CoreServerConfig;
pub use error::CoreError;
pub use log::init_log;
use ppaass_2025_crypto::{generate_aes_encryption_token, generate_blowfish_encryption_token, RsaCrypto};
use ppaass_2025_protocol::Encryption;
use rand::random;
pub use runtime::generate_core_runtime;
pub use server::CoreServer;
pub use server::CoreServerGuard;
pub use server::CoreServerState;
use std::borrow::Cow;
/// Randomly generate a raw encryption
#[inline(always)]
pub fn random_generate_encryption() -> Encryption {
    let random_number = random::<u64>();
    if random_number % 2 == 0 {
        Encryption::Aes(generate_aes_encryption_token())
    } else {
        Encryption::Blowfish(generate_blowfish_encryption_token())
    }
}
#[inline(always)]
pub fn rsa_encrypt_encryption<'a>(
    raw_encryption: &'a Encryption,
    rsa_crypto: &RsaCrypto,
) -> Result<Cow<'a, Encryption>, CoreError> {
    match raw_encryption {
        Encryption::Plain => Ok(Cow::Borrowed(raw_encryption)),
        Encryption::Aes(token) => {
            let encrypted_token = rsa_crypto.encrypt(&token)?;
            Ok(Cow::Owned(Encryption::Aes(encrypted_token)))
        }
        Encryption::Blowfish(token) => {
            let encrypted_token = rsa_crypto.encrypt(&token)?;
            Ok(Cow::Owned(Encryption::Blowfish(encrypted_token)))
        }
    }
}
#[inline(always)]
pub fn rsa_decrypt_encryption<'a>(
    encrypted_encryption: &'a Encryption,
    rsa_crypto: &RsaCrypto,
) -> Result<Cow<'a, Encryption>, CoreError> {
    match encrypted_encryption {
        Encryption::Plain => Ok(Cow::Borrowed(encrypted_encryption)),
        Encryption::Aes(token) => {
            let decrypted_token = rsa_crypto.decrypt(&token)?;
            Ok(Cow::Owned(Encryption::Aes(decrypted_token)))
        }
        Encryption::Blowfish(token) => {
            let decrypted_token = rsa_crypto.decrypt(&token)?;
            Ok(Cow::Owned(Encryption::Blowfish(decrypted_token)))
        }
    }
}
