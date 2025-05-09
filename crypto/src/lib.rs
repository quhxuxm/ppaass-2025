mod aes;
mod blowfish;
mod error;
mod rsa;
pub use aes::*;
pub use blowfish::*;
pub use error::Error;
use rand::random;
pub use rsa::*;
#[inline(always)]
fn random_n_bytes<const N: usize>() -> Vec<u8> {
    let random_n_bytes = random::<[u8; N]>();
    random_n_bytes.to_vec()
}
