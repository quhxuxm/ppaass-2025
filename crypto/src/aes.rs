use crate::error::Error;
use crate::random_n_bytes;
use aes::Aes256;
use bytes::Bytes;
use cipher::block_padding::Pkcs7;
use cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
type Aes256CbcEncryptor = cbc::Encryptor<Aes256>;
type Aes256CbcDecryptor = cbc::Decryptor<Aes256>;
/// Generate the encryption token for AES
/// The first 32 bytes is the key
/// The last 16 bytes is the iv
#[inline(always)]
pub fn generate_aes_encryption_token() -> Vec<u8> {
    random_n_bytes::<48>()
}
/// Encrypt the target bytes with AES
#[inline(always)]
pub fn encrypt_with_aes(encryption_token: &[u8], target: &[u8]) -> Result<Bytes, Error> {
    let aes_encryptor =
        Aes256CbcEncryptor::new_from_slices(&encryption_token[..32], &encryption_token[32..])?;
    Ok(aes_encryptor.encrypt_padded_vec_mut::<Pkcs7>(target).into())
}
/// Decrypt the target bytes with AES
#[inline(always)]
pub fn decrypt_with_aes(encryption_token: &[u8], target: &[u8]) -> Result<Bytes, Error> {
    let aes_decrypt =
        Aes256CbcDecryptor::new_from_slices(&encryption_token[..32], &encryption_token[32..])?;
    let result = aes_decrypt.decrypt_padded_vec_mut::<Pkcs7>(target)?;
    Ok(result.into())
}
#[test]
fn test() -> Result<(), Error> {
    let encryption_token = generate_aes_encryption_token();
    let target = "hello world! this is my plaintext.".as_bytes().to_vec();
    let data_len = target.len();
    println!("Data length: {}", data_len);
    let encrypt_result = encrypt_with_aes(&encryption_token, &target)?;
    println!(
        "Encrypt result: [{:?}]",
        String::from_utf8_lossy(&encrypt_result)
    );
    let encrypt_result = encrypt_result.to_vec();
    let decrypted_result = decrypt_with_aes(&encryption_token, &encrypt_result)?;
    println!(
        "Decrypted result: [{:?}]",
        String::from_utf8_lossy(&decrypted_result)
    );
    Ok(())
}
