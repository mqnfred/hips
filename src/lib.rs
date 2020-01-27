use ::std::collections::BTreeMap;
use ::std::io::Write;

pub trait Store {
    fn set(&mut self, key: String, value: String) -> Result<(), ::failure::Error>;
    fn get(&self, key: String) -> Result<String, ::failure::Error>;
    fn all(&self) -> Result<BTreeMap<String, String>, ::failure::Error>;
}

pub struct YAMLStore {
    path: String,
    password: String,
}

impl Store for YAMLStore {
    fn set(&mut self, key: String, value: String) -> Result<(), ::failure::Error> {
        let mut map = self.read()?;
        map.insert(key, value);
        self.write(map)
    }

    fn get(&self, key: String) -> Result<String, ::failure::Error> {
        self.read()?.get(&key).map(|f| {
            Ok(f.to_owned())
        }).unwrap_or_else(|| Err(::failure::err_msg("key not found")))
    }

    fn all(&self) -> Result<BTreeMap<String, String>, ::failure::Error> {
        self.read()
    }
}

impl YAMLStore {
    pub fn new(path: String, password: String) -> Self {
        Self { path, password }
    }

    fn read(&self) -> Result<BTreeMap<String, String>, ::failure::Error> {
        Ok(::serde_yaml::from_str(&match ::std::fs::read_to_string(&self.path) {
            Err(err) if err.kind() == ::std::io::ErrorKind::NotFound => Ok("{}".to_string()),
            Err(err) => Err(err),
            Ok(val) => Ok(val),
        }?)?)
    }

    fn write(&self, map: BTreeMap<String, String>) -> Result<(), ::failure::Error> {
        let yaml = serde_yaml::to_string(&map)?;
        let mut file = ::std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(&self.path)?;
        file.write_all(yaml.as_bytes())?;
        Ok(())
    }
}
