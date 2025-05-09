use crate::error::Error;
use crate::random_n_bytes;
use blowfish::Blowfish;
use bytes::Bytes;
use cipher::block_padding::Pkcs7;
use cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
type BlowfishCbcEncryptor = cbc::Encryptor<Blowfish>;
type BlowfishCbcDecryptor = cbc::Decryptor<Blowfish>;
/// Generate the encryption token for Blowfish
/// The first 56 bytes is the key
/// The last 8 bytes is the iv
#[inline(always)]
pub fn generate_blowfish_encryption_token() -> Vec<u8> {
    random_n_bytes::<64>()
}
/// Encrypt the target bytes with Blowfish
#[inline(always)]
pub fn encrypt_with_blowfish(encryption_token: &[u8], target: &[u8]) -> Result<Bytes, Error> {
    let encryptor =
        BlowfishCbcEncryptor::new_from_slices(&encryption_token[..56], &encryption_token[56..])?;
    let result = encryptor.encrypt_padded_vec_mut::<Pkcs7>(target);
    Ok(result.into())
}
/// Decrypt the target bytes with Blowfish
#[inline(always)]
pub fn decrypt_with_blowfish(encryption_token: &[u8], target: &[u8]) -> Result<Bytes, Error> {
    let decryptor =
        BlowfishCbcDecryptor::new_from_slices(&encryption_token[..56], &encryption_token[56..])?;
    let result = decryptor.decrypt_padded_vec_mut::<Pkcs7>(target)?;
    Ok(result.into())
}
#[test]
fn test() -> Result<(), Error> {
    let encryption_token = generate_blowfish_encryption_token();
    let target = "hello world! this is my plaintext.".as_bytes().to_vec();
    let encrypt_result = encrypt_with_blowfish(&encryption_token, &target)?;
    println!(
        "Encrypt result: [{:?}]",
        String::from_utf8_lossy(&encrypt_result)
    );
    let decrypted_result = decrypt_with_blowfish(&encryption_token, &encrypt_result)?;
    println!(
        "Decrypted result: [{:?}]",
        String::from_utf8_lossy(&decrypted_result)
    );
    Ok(())
}
