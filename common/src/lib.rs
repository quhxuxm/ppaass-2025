mod codec;
mod config;
mod error;
mod log;
mod runtime;
mod server;
pub use codec::SecureLengthDelimitedCodec;
pub use config::BaseLogConfig;
pub use config::BaseRuntimeConfig;
pub use config::BaseServerConfig;
pub use error::BaseError;
pub use log::init_log;
use ppaass_2025_crypto::{
    generate_aes_encryption_token, generate_blowfish_encryption_token, RsaCrypto,
};
use ppaass_2025_protocol::Encryption;
use rand::random;
pub use runtime::generate_base_runtime;
pub use server::BaseServer;
pub use server::BaseServerGuard;
pub use server::BaseServerState;
use std::borrow::Cow;
use std::sync::LazyLock;
pub const HANDSHAKE_ENCRYPTION: LazyLock<Encryption> = LazyLock::new(|| {
    Encryption::Blowfish({
        b"1212398347384737434783748347387438743742982332672763272320119203".to_vec()
    })
});

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
) -> Result<Cow<'a, Encryption>, BaseError> {
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
) -> Result<Cow<'a, Encryption>, BaseError> {
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
