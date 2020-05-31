use crate::prelude::*;

pub struct OpenSSL {
    password: String,
}
impl OpenSSL {
    pub fn new(password: String) -> Self {
        Self { password }
    }
}
impl Encrypter for OpenSSL {
    fn encrypt(&mut self, secret: Secret) -> Result<Encrypted> {
        let mut iv = vec![0u8; IV_SIZE];
        ::openssl::rand::rand_bytes(&mut iv).context("generating random bytes")?;
        let mut salt = vec![0u8; SALT_SIZE];
        ::openssl::rand::rand_bytes(&mut salt).context("generating random bytes")?;
        let mut tag = vec![0u8; TAG_SIZE];

        let ciphertext = ::openssl::symm::encrypt_aead(
            ::openssl::symm::Cipher::aes_256_gcm(),
            &self.key(&salt).context("generating key")?,
            Some(&iv), &[], secret.secret.as_bytes(), &mut tag,
        ).context("encrypting plaintext")?;

        let ciphertext = ::base64::encode(&iv.into_iter().chain(
            tag.into_iter()
        ).chain(ciphertext.into_iter()).collect::<Vec<u8>>());
        let salt = ::base64::encode(&salt);

        Ok(Encrypted{ name: secret.name, secret: ciphertext, salt })
    }

    fn decrypt(&mut self, encrypted: Encrypted) -> Result<Secret> {
        let ciphertext = ::base64::decode(&encrypted.secret).context("decoding ciphertext")?;

        let iv = &ciphertext[..IV_SIZE];
        let tag = &ciphertext[IV_SIZE..(IV_SIZE + TAG_SIZE)];
        let ciphertext = &ciphertext[(IV_SIZE + TAG_SIZE)..];
        let salt = ::base64::decode(&encrypted.salt).context("decoding salt")?;

        let secret = ::openssl::symm::decrypt_aead(
            ::openssl::symm::Cipher::aes_256_gcm(),
            &self.key(&salt).context("generating key")?,
            Some(&iv), &[], &ciphertext, &tag,
        ).context("processing ciphertext")?;

        Ok(Secret{
            name: encrypted.name,
            secret: ::std::str::from_utf8(&secret).context("loading as utf8")?.to_owned(),
        })
    }
}
impl OpenSSL {
    fn key(&self, salt: &[u8]) -> Result<Vec<u8>> {
        let mut pbkdf2_hash = [0u8; KEY_LEN];
        ::openssl::pkcs5::pbkdf2_hmac(
            self.password.as_bytes(),
            &salt,
            ITERATIONS,
            ::openssl::hash::MessageDigest::sha256(),
            &mut pbkdf2_hash,
        )?;
        Ok(pbkdf2_hash.to_vec())
    }
}
const IV_SIZE: usize = 12;
const SALT_SIZE: usize = 32;
const TAG_SIZE: usize = 16;
const KEY_LEN: usize = 32;
const ITERATIONS: usize = 100_000;
