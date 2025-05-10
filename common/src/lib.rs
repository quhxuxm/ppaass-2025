mod codec;
pub mod config;
mod error;
mod log;
pub mod proxy;
mod runtime;
mod server;
pub mod user;
pub use codec::SecureLengthDelimitedCodec;
pub use config::WithLogConfig;
pub use config::WithServerConfig;
pub use config::WithServerRuntimeConfig;
use crypto::{RsaCrypto, generate_aes_encryption_token, generate_blowfish_encryption_token};
pub use error::Error;
pub use log::init_log;
use protocol::Encryption;
use rand::random;
pub use runtime::build_server_runtime;
pub use server::ServerGuard;
pub use server::ServerState;
pub use server::start_server;
use std::borrow::Cow;
use std::sync::LazyLock;
static HANDSHAKE_ENCRYPTION: LazyLock<Encryption> = LazyLock::new(|| {
    Encryption::Blowfish({
        b"1212398347384737434783748347387438743742982332672763272320119203".to_vec()
    })
});

pub fn get_handshake_encryption() -> &'static Encryption {
    &HANDSHAKE_ENCRYPTION
}

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
) -> Result<Cow<'a, Encryption>, Error> {
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
) -> Result<Cow<'a, Encryption>, Error> {
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
