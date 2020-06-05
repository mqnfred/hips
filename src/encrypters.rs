use crate::prelude::*;
use ring::rand::SecureRandom;
use std::convert::TryInto;

const IV_SIZE: usize = 12;
const SALT_SIZE: usize = 32;
const TAG_SIZE: usize = 16;
const KEY_LEN: usize = 32;
const ITERATIONS: usize = 100_000;

pub struct Ring(String);

impl Ring {
    pub fn new(password: String) -> Self {
        Self(password)
    }

    fn key(&self, salt: &[u8]) -> Result<Vec<u8>> {
        let mut key = [0; KEY_LEN];
        ::ring::pbkdf2::derive(
            ::ring::pbkdf2::PBKDF2_HMAC_SHA256,
            ::core::num::NonZeroU32::new(ITERATIONS as u32).expect("iterations not 0"),
            &salt,
            self.0.as_bytes(),
            &mut key,
        );
        Ok(key.to_vec())
    }
}

impl Encrypter for Ring {
    fn encrypt(&mut self, secret: Secret) -> Result<Encrypted> {
        assert_eq!(::ring::aead::AES_256_GCM.tag_len(), TAG_SIZE);

        let plaintext = ::ring::aead::Aad::empty();
        let mut ciphertext = secret.secret.as_bytes().to_vec();
        for _ in 0..::ring::aead::AES_256_GCM.tag_len() {
            ciphertext.push(0);
        }

        let rand = ::ring::rand::SystemRandom::new();
        let mut salt = vec![0u8; SALT_SIZE];
        rand.fill(&mut salt)
            .map_err(|err| Error::msg(format!("{}", err)))?;
        let mut iv = [0u8; IV_SIZE];
        rand.fill(&mut iv)
            .map_err(|err| Error::msg(format!("{}", err)))?;
        let nonce = ::ring::aead::Nonce::assume_unique_for_key(iv);

        let key = self.key(&salt)?;
        let key = ::ring::aead::UnboundKey::new(&::ring::aead::AES_256_GCM, &key[..])
            .map_err(|err| Error::msg(err.to_string()))?;
        let key = ::ring::aead::LessSafeKey::new(key);

        key.seal_in_place_append_tag(nonce, plaintext, &mut ciphertext)
            .map_err(|err| Error::msg(err.to_string()))?;

        let ciphertext = ::base64::encode(
            &iv.iter()
                .chain(ciphertext.iter())
                .map(|v| *v)
                .collect::<Vec<u8>>(),
        );
        let salt = ::base64::encode(&salt);

        Ok(Encrypted {
            name: secret.name,
            secret: ciphertext,
            salt,
        })
    }

    fn decrypt(&mut self, encrypted: Encrypted) -> Result<Secret> {
        let plaintext = ::ring::aead::Aad::empty();
        let mut ciphertext = ::base64::decode(&encrypted.secret).context("decoding ciphertext")?;

        let iv = &ciphertext[..IV_SIZE];
        let nonce =
            ::ring::aead::Nonce::assume_unique_for_key(iv.try_into().context("transforming iv")?);
        let mut ciphertext = &mut ciphertext[IV_SIZE..];
        let salt = ::base64::decode(&encrypted.salt).context("decoding salt")?;

        let key = self.key(&salt).context("computing key")?;
        let key = ::ring::aead::UnboundKey::new(&::ring::aead::AES_256_GCM, &key[..])
            .map_err(|err| Error::msg(err.to_string()))
            .context("generating unbound key")?;
        let key = ::ring::aead::LessSafeKey::new(key);

        let secret = key
            .open_in_place(nonce, plaintext, &mut ciphertext)
            .map_err(|err| Error::msg(err.to_string()))
            .context("running aes_256_gcm")?;

        Ok(Secret {
            name: encrypted.name,
            secret: ::std::str::from_utf8(&secret[..(secret.len() - TAG_SIZE)])
                .context("loading as utf8")?
                .to_owned(),
        })
    }
}
