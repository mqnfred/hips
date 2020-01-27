#[macro_use]
extern crate magic_crypt;

use ::std::collections::BTreeMap;
use ::std::io::Write;

pub trait Store {
    fn set(&mut self, key: String, value: String) -> Result<(), ::failure::Error>;
    fn get(&mut self, key: String) -> Result<String, ::failure::Error>;
    fn all(&mut self) -> Result<BTreeMap<String, String>, ::failure::Error>;
}

pub struct YAMLStore<E: Encrypter> {
    path: String,
    encrypter: E,
}

impl<E: Encrypter> Store for YAMLStore<E> {
    fn set(&mut self, key: String, value: String) -> Result<(), ::failure::Error> {
        let mut map = self.read()?;
        map.insert(key, value);
        self.write(map)
    }

    fn get(&mut self, key: String) -> Result<String, ::failure::Error> {
        self.read()?.get(&key).map(|f| {
            Ok(f.to_owned())
        }).unwrap_or_else(|| Err(::failure::err_msg("key not found")))
    }

    fn all(&mut self) -> Result<BTreeMap<String, String>, ::failure::Error> {
        self.read()
    }
}

impl<E: Encrypter> YAMLStore<E> {
    pub fn new(path: String, password: String) -> Self {
        Self { path, encrypter: E::new(password) }
    }

    fn read(&mut self) -> Result<BTreeMap<String, String>, ::failure::Error> {
        let mut encrypted_map: BTreeMap<String, String> = ::serde_yaml::from_str(
            &match ::std::fs::read_to_string(&self.path) {
                Err(err) if err.kind() == ::std::io::ErrorKind::NotFound => Ok("{}".to_string()),
                Err(err) => Err(err),
                Ok(val) => Ok(val),
            }?,
        )?;

        let mut map = BTreeMap::<String, String>::new();
        for (k, v) in encrypted_map.into_iter() {
            map.insert(k, self.encrypter.decrypt(&v)?);
        }
        Ok(map)
    }

    fn write(&mut self, mut map: BTreeMap<String, String>) -> Result<(), ::failure::Error> {
        let mut encrypted_map = BTreeMap::<String, String>::new();
        for (k, v) in map.into_iter() {
            encrypted_map.insert(k, self.encrypter.encrypt(&v)?);
        }

        let yaml = serde_yaml::to_string(&encrypted_map)?;
        let mut file = ::std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(&self.path)?;
        file.write_all(yaml.as_bytes())?;
        Ok(())
    }
}

pub trait Encrypter {
    fn new(password: String) -> Self;
    fn encrypt(&mut self, value: &str) -> Result<String, ::failure::Error>;
    fn decrypt(&mut self, value: &str) -> Result<String, ::failure::Error>;
}

pub struct MagicEncrypter(::magic_crypt::MagicCrypt);

impl Encrypter for MagicEncrypter {
    fn new(key: String) -> Self {
        Self(::magic_crypt::new_magic_crypt!(&key, 256))
    }

    fn encrypt(&mut self, value: &str) -> Result<String, ::failure::Error> {
        Ok(self.0.encrypt_str_to_base64(value))
    }

    fn decrypt(&mut self, value: &str) -> Result<String, ::failure::Error> {
        Ok(self.0.decrypt_base64_to_string(value).map_err(|err| {
            ::failure::err_msg("failed_to_decrypt")
        })?)
    }
}
