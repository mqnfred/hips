use crate::prelude::*;

impl Database {
    pub fn new(path: PathBuf, password: String) -> Result<Self> {
        if let Some(extension) = path.extension() {
            if extension == "yaml" {
                Ok(Database{
                    b: Box::new(crate::backends::YAML::new(path)),
                    e: Box::new(crate::encrypters::OpenSSL::new(password)),
                })
            } else {
                Err(Error::msg(format!("unsupported format: {}", extension.to_str().unwrap())))
            }
        } else {
            Ok(Database{
                b: Box::new(crate::backends::Folder::new(path)),
                e: Box::new(crate::encrypters::OpenSSL::new(password)),
            })
        }
    }
}
impl Database {
    pub fn set(&mut self, secret: Secret) -> Result<()> {
        let encrypted = self.e.encrypt(secret).context("encrypting secret")?;
        self.b.set(encrypted).context("storing secret")
    }

    pub fn get(&mut self, name: String) -> Result<Secret> {
        self.e.decrypt(
            self.b.get(name).context("looking up name")?
        ).context("decrypting secret")
    }

    pub fn remove(&mut self, name: String) -> Result<()> {
        self.b.remove(name).context("removing secret")
    }

    pub fn list(&mut self) -> Result<Vec<Secret>> {
        self.b.list().context("listing secrets")?.into_iter().map(|s| {
            self.e.decrypt(s)
        }).collect::<Result<Vec<Secret>>>()
    }
}
