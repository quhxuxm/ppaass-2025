mod aes;
mod blowfish;
mod rsa;
mod error;
pub use aes::*;
pub use blowfish::*;
use bytes::Bytes;
pub use error::CryptoError;
use rand::random;
pub use rsa::*;
#[inline(always)]
fn random_n_bytes<const N: usize>() -> Bytes {
    let random_n_bytes = random::<[u8; N]>();
    random_n_bytes.to_vec().into()
}
