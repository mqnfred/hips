use ::anyhow::{Context,Error};
use ::std::collections::BTreeMap;
use ::std::io::Write;

pub type EncryptedYaml = stores::EncryptedStore<stores::YAML, encrypters::OpenSSL>;

pub trait Store {
    fn set(&mut self, name: String, value: String) -> Result<(), Error>;
    fn get(&mut self, name: String) -> Result<String, Error>;
    fn all(&mut self) -> Result<BTreeMap<String, String>, Error>;
}

pub trait Encrypter {
    fn new(password: String) -> Self;
    fn encrypt(&mut self, value: &str) -> Result<String, Error>;
    fn decrypt(&mut self, value: &str) -> Result<String, Error>;
}

mod stores {
    use super::*;

    pub struct EncryptedStore<S: Store, E: Encrypter>(S, E);
    impl EncryptedStore<YAML, encrypters::OpenSSL> {
        pub fn new(path: String, password: String) -> Self {
            Self(YAML::new(path), encrypters::OpenSSL::new(password))
        }
    }
    impl<S: Store, E: Encrypter> Store for EncryptedStore<S, E> {
        fn set(&mut self, name: String, value: String) -> Result<(), Error> {
            self.0.set(
                self.1.encrypt(&name).context("encrypting name")?,
                self.1.encrypt(&value).context("encrypting secret")?,
            )
        }

        fn get(&mut self, name: String) -> Result<String, Error> {
            let encrypted_name = self.1.encrypt(&name).context("encrypting name")?;
            self.1.decrypt(
                &self.0.get(encrypted_name).context("looking up name")?
            ).context("decrypting secret")
        }

        fn all(&mut self) -> Result<BTreeMap<String, String>, Error> {
            let mut map = BTreeMap::<String, String>::new();
            for (k, v) in self.0.all().context("listing secrets")? {
                map.insert(
                    self.1.decrypt(&k).context("decrypting name")?,
                    self.1.decrypt(&v).context("decrypting secret")?,
                );
            }
            Ok(map)
        }
    }

    pub struct YAML {
        path: String,
    }
    impl YAML {
        pub fn new(path: String) -> Self {
            Self { path }
        }

        fn read(&mut self) -> Result<BTreeMap<String, String>, Error> {
            Ok(::serde_yaml::from_str(&match ::std::fs::read_to_string(&self.path) {
                Err(err) if err.kind() == ::std::io::ErrorKind::NotFound => Ok("{}".to_string()),
                Err(err) => Err(err),
                Ok(val) => Ok(val),
            }.context("reading file")?).context("unmarshalling yaml")?)
        }

        fn write(&mut self, map: BTreeMap<String, String>) -> Result<(), Error> {
            let mut f = ::std::fs::OpenOptions::new().write(true).create(true)
                .truncate(true).open(&self.path).context("opening file")?;
            Ok(f.write_all(
                ::serde_yaml::to_string(&map).context("marshalling to yaml")?.as_bytes()
            ).context("writing to file")?)
        }
    }
    impl Store for YAML {
        fn set(&mut self, name: String, value: String) -> Result<(), Error> {
            let mut map = self.read().context("loading database")?;
            map.insert(name, value);
            self.write(map).context("writing database")
        }

        fn get(&mut self, name: String) -> Result<String, Error> {
            self.read().context("loading database")?.get(&name).map(|f| {
                Ok(f.to_owned())
            }).unwrap_or_else(|| Err(Error::msg(format!("secret not found: {}", name))))
        }

        fn all(&mut self) -> Result<BTreeMap<String, String>, Error> {
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

        fn encrypt(&mut self, to_encrypt: &str) -> Result<String, Error> {
            let mut iv = vec![0u8; IV_SIZE];
            ::openssl::rand::rand_bytes(&mut iv).context("generating random bytes")?;
            let mut salt = vec![0u8; SALT_SIZE];
            ::openssl::rand::rand_bytes(&mut salt).context("generating random bytes")?;
            let mut tag = vec![0u8; TAG_SIZE];

            let cipher = ::openssl::symm::Cipher::aes_256_gcm();
            let ciphertext = ::openssl::symm::encrypt_aead(
                cipher, &self.key(&salt).context("generating key")?,
                Some(&iv), &[], to_encrypt.as_bytes(), &mut tag,
            ).context("encrypting plaintext")?;

            Ok(::base64::encode(
                &iv.into_iter().chain(
                    salt.into_iter().chain(tag.into_iter())
                ).chain(ciphertext.into_iter()).collect::<Vec<u8>>()
            ))
        }

        fn decrypt(&mut self, to_decrypt: &str) -> Result<String, Error> {
            let to_decrypt = ::base64::decode(to_decrypt).context("decoding base64")?;
            let iv = &to_decrypt[..IV_SIZE];
            let salt = &to_decrypt[IV_SIZE..(IV_SIZE + SALT_SIZE)];
            let tag = &to_decrypt[(IV_SIZE + SALT_SIZE)..(IV_SIZE + SALT_SIZE + TAG_SIZE)];
            let ciphertext = &to_decrypt[(IV_SIZE + SALT_SIZE + TAG_SIZE)..];
            Ok(::std::str::from_utf8(&::openssl::symm::decrypt_aead(
                ::openssl::symm::Cipher::aes_256_gcm(),
                &self.key(salt).context("generating key")?,
                Some(&iv), &[], &ciphertext, &tag,
            ).context("processing ciphertext")?).context("loading string as utf8")?.into())
        }
    }
}
