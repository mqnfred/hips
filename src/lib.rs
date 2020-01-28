use ::failure::Error;
use ::std::collections::BTreeMap;
use ::std::io::Write;

pub type EncryptedYaml = stores::EncryptedStore<stores::YAML, encrypters::Magic>;

pub trait Store {
    fn set(&mut self, key: String, value: String) -> Result<(), Error>;
    fn get(&mut self, key: String) -> Result<String, Error>;
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

    impl EncryptedStore<YAML, encrypters::Magic> {
        pub fn new(path: String, password: String) -> Self {
            Self(YAML::new(path), encrypters::Magic::new(password))
        }
    }

    impl<S: Store, E: Encrypter> Store for EncryptedStore<S, E> {
        fn set(&mut self, key: String, value: String) -> Result<(), Error> {
            self.0.set(key, self.1.encrypt(&value)?)
        }

        fn get(&mut self, key: String) -> Result<String, Error> {
            self.1.decrypt(&self.0.get(key)?)
        }

        fn all(&mut self) -> Result<BTreeMap<String, String>, Error> {
            let mut map = BTreeMap::<String, String>::new();
            for (k, v) in self.0.all()? {
                map.insert(k, self.1.decrypt(&v)?);
            }
            Ok(map)
        }
    }

    pub struct YAML {
        path: String,
    }

    impl Store for YAML {
        fn set(&mut self, key: String, value: String) -> Result<(), Error> {
            let mut map = self.read()?;
            map.insert(key, value);
            self.write(map)
        }

        fn get(&mut self, key: String) -> Result<String, Error> {
            self.read()?.get(&key).map(|f| {
                Ok(f.to_owned())
            }).unwrap_or_else(|| Err(::failure::err_msg("key not found")))
        }

        fn all(&mut self) -> Result<BTreeMap<String, String>, Error> {
            self.read()
        }
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
            }?)?)
        }

        fn write(&mut self, map: BTreeMap<String, String>) -> Result<(), Error> {
            let mut f = ::std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(&self.path)?;
            Ok(f.write_all(::serde_yaml::to_string(&map)?.as_bytes())?)
        }
    }
}

mod encrypters {
    use super::*;

    pub struct Magic(::magic_crypt::MagicCrypt);

    impl Encrypter for Magic {
        fn new(key: String) -> Self {
            Self(::magic_crypt::new_magic_crypt!(&key, 256))
        }

        fn encrypt(&mut self, value: &str) -> Result<String, Error> {
            Ok(self.0.encrypt_str_to_base64(value))
        }

        fn decrypt(&mut self, value: &str) -> Result<String, Error> {
            Ok(self.0.decrypt_base64_to_string(value).map_err(|_| {
                ::failure::err_msg("failed_to_decrypt")
            })?)
        }
    }
}
