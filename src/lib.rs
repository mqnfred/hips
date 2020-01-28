#![feature(trait_alias)]

#[macro_use]
extern crate serde;

use ::anyhow::{Context,Error};
use ::std::io::Write;

pub trait SecretStore = Store<Secret>;
pub type EncryptedYaml = stores::EncryptedStore<stores::YAML, encrypters::OpenSSL>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Secret {
    pub name: String,
    pub secret: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encrypted {
    name: String,
    secret: String,
    salt: String,
}

pub trait Store<FORMAT: ::serde::Serialize + for<'a> ::serde::Deserialize<'a>> {
    fn set(&mut self, payload: FORMAT) -> Result<(), Error>;
    fn get(&mut self, name: String) -> Result<FORMAT, Error>;
    fn all(&mut self) -> Result<Vec<FORMAT>, Error>;
}

pub trait Encrypter {
    fn new(password: String) -> Self;
    fn encrypt(&mut self, secret: Secret) -> Result<Encrypted, Error>;
    fn decrypt(&mut self, encrypted: Encrypted) -> Result<Secret, Error>;
}

mod stores {
    use super::*;

    pub struct EncryptedStore<S: Store<Encrypted>, E: Encrypter>(S, E);
    impl EncryptedStore<YAML, encrypters::OpenSSL> {
        pub fn new(path: String, password: String) -> Self {
            Self(YAML::new(path), encrypters::OpenSSL::new(password))
        }
    }
    impl<S: Store<Encrypted>, E: Encrypter> Store<Secret> for EncryptedStore<S, E> {
        fn set(&mut self, secret: Secret) -> Result<(), Error> {
            let encrypted = self.1.encrypt(secret).context("encrypting secret")?;
            self.0.set(encrypted).context("storing secret")
        }

        fn get(&mut self, name: String) -> Result<Secret, Error> {
            self.1.decrypt(
                self.0.get(name).context("looking up name")?
            ).context("decrypting secret")
        }

        fn all(&mut self) -> Result<Vec<Secret>, Error> {
            self.0.all().context("listing secrets")?.into_iter().map(|s| {
                self.1.decrypt(s)
            }).collect::<Result<Vec<Secret>, Error>>()
        }
    }

    pub struct YAML {
        path: String,
    }
    impl YAML {
        pub fn new(path: String) -> Self {
            Self { path }
        }

        fn read(&mut self) -> Result<Vec<Encrypted>, Error> {
            Ok(::serde_yaml::from_str(&match ::std::fs::read_to_string(&self.path) {
                Err(err) if err.kind() == ::std::io::ErrorKind::NotFound => Ok("[]".to_string()),
                Err(err) => Err(err),
                Ok(val) => Ok(val),
            }.context("reading file")?).context("unmarshalling yaml")?)
        }

        fn write(&mut self, secrets: Vec<Encrypted>) -> Result<(), Error> {
            let mut f = ::std::fs::OpenOptions::new().write(true).create(true)
                .truncate(true).open(&self.path).context("opening file")?;
            Ok(f.write_all(
                ::serde_yaml::to_string(&secrets).context("marshalling to yaml")?.as_bytes()
            ).context("writing to file")?)
        }
    }
    impl Store<Encrypted> for YAML {
        fn set(&mut self, encrypted: Encrypted) -> Result<(), Error> {
            let mut secrets = self.read().context("loading database")?;
            if let Some(existing_pos) = secrets.iter().position(|s| s.name == encrypted.name) {
                secrets.remove(existing_pos);
                secrets.insert(existing_pos, encrypted);
            } else {
                secrets.push(encrypted);
            }
            self.write(secrets).context("writing database")
        }

        fn get(&mut self, name: String) -> Result<Encrypted, Error> {
            self.read().context("loading database")?.into_iter().find(|s| {
                s.name == name
            }).map(|s| Ok(s)).unwrap_or_else(|| Err(Error::msg("secret not found")))
        }

        fn all(&mut self) -> Result<Vec<Encrypted>, Error> {
            self.read()
        }
    }
}

mod encrypters {
    use super::*;

    const IV_SIZE: usize = 12;
    const SALT_SIZE: usize = 32;
    const TAG_SIZE: usize = 16;
    const KEY_LEN: usize = 32;
    const ITERATIONS: usize = 100_000;
    pub struct OpenSSL {
        password: String,
    }
    impl OpenSSL {
        fn key(&self, salt: &[u8]) -> Result<Vec<u8>, Error> {
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
    impl Encrypter for OpenSSL {
        fn new(password: String) -> Self {
            Self { password }
        }

        fn encrypt(&mut self, secret: Secret) -> Result<Encrypted, Error> {
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

        fn decrypt(&mut self, encrypted: Encrypted) -> Result<Secret, Error> {
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
}
