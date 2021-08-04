//! [`Backend`][1] trait implementations.
//!
//! [1]: ../trait.Backend.html

use crate::prelude::*;

/// Store secrets in yaml format.
///
/// We store a list of encrypted secrets, each entry containing:
///
///  - name
///  - secret (encrypted, base64)
///  - salt (base64)
///
/// This `Backend` will be selected by the binary if the given database path ends with `.yaml`.
pub struct YAML {
    path: PathBuf,
}

impl YAML {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
impl Backend for YAML {
    fn store(&mut self, encrypted: Encrypted) -> Result<()> {
        let mut secrets = self.read().context("loading database")?;
        if let Some(existing_pos) = secrets.iter().position(|s| s.name == encrypted.name) {
            secrets.remove(existing_pos);
            secrets.insert(existing_pos, encrypted);
        } else {
            secrets.push(encrypted);
        }
        self.write(secrets).context("writing database")
    }

    fn load(&self, name: String) -> Result<Encrypted> {
        self.read().context("loading database")?.into_iter().find(|s| {
            s.name == name
        }).map(|s| Ok(s)).unwrap_or_else(|| Err(Error::msg("secret not found")))
    }

    fn remove(&mut self, name: String) -> Result<()> {
        let secrets = self.read().context("loading database")?.into_iter().filter(|s| {
            s.name != name
        }).collect();
        self.write(secrets)
    }

    fn list(&self) -> Result<Vec<Encrypted>> {
        self.read()
    }
}
impl YAML {
    fn read(&self) -> Result<Vec<Encrypted>> {
        Ok(::serde_yaml::from_str(&match ::std::fs::read_to_string(&self.path) {
            Err(err) if err.kind() == ::std::io::ErrorKind::NotFound => Ok("[]".to_string()),
            Err(err) => Err(err),
            Ok(val) => Ok(val),
        }.context("reading file")?).context("unmarshalling yaml")?)
    }

    fn write(&mut self, secrets: Vec<Encrypted>) -> Result<()> {
        let mut f = ::std::fs::OpenOptions::new().write(true).create(true)
            .truncate(true).open(&self.path).context("opening file")?;
        Ok(f.write_all(
            ::serde_yaml::to_string(&secrets).context("marshalling to yaml")?.as_bytes()
        ).context("writing to file")?)
    }
}

/// Store the secrets in a directory hierarchy.
///
/// The path points to the main folder, which is created by the library. A sub-folder named after
/// the secret entry is created for each secret, with the following files inside it:
///
///  - secret (encrypted, base64)
///  - salt (base64)
///
/// This `Backend` will be selected by the binary if the given database path has no extension.
pub struct Folder {
    path: PathBuf,
}
impl Folder {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
impl Backend for Folder {
    fn store(&mut self, encrypted: Encrypted) -> Result<()> {
        self.ensure_root(&encrypted.name)?;

        let mut salt_f = ::std::fs::OpenOptions::new().write(true).create(true)
            .truncate(true).open(self.salt_path(&encrypted.name)).context("opening file")?;
        salt_f.write_all(encrypted.salt.as_bytes())?;

        let mut secret_f = ::std::fs::OpenOptions::new().write(true).create(true)
            .truncate(true).open(self.secret_path(&encrypted.name)).context("opening file")?;
        Ok(secret_f.write_all(encrypted.secret.as_bytes())?)
    }

    fn load(&self, name: String) -> Result<Encrypted> {
        let salt_path = self.salt_path(&name);
        let secret_path = self.secret_path(&name);
        Ok(Encrypted{
            name,
            secret: ::std::fs::read_to_string(secret_path).context("reading secret file")?,
            salt: ::std::fs::read_to_string(salt_path).context("reading salt file")?,
        })
    }

    fn remove(&mut self, name: String) -> Result<()> {
        Ok(::std::fs::remove_dir_all(self.path.join(name))?)
    }

    fn list(&self) -> Result<Vec<Encrypted>> {
        ::std::fs::read_dir(&self.path).context(
            "listing secret files"
        )?.collect::<Result<Vec<_>, _>>()?.into_iter().filter_map(|dir| {
            dir.path().file_name().map(|fname| self.load(fname.to_str().unwrap().to_owned()))
        }).collect::<Result<Vec<_>>>()
    }
}
impl Folder {
    fn ensure_root(&self, name: &str) -> Result<PathBuf> {
        let root_path = self.path.join(name);
        let root_md = match ::std::fs::metadata(&root_path) {
            Err(err) if err.kind() == ::std::io::ErrorKind::NotFound => {
                ::std::fs::create_dir_all(&root_path)?;
                ::std::fs::metadata(&root_path)?
            }
            Err(err) => return Err(err.into()),
            Ok(md) => md,
        };

        if root_md.is_dir() {
            Ok(root_path)
        } else {
            Err(Error::msg("secret path is invalid (should be a directory)"))
        }
    }

    fn salt_path(&self, name: &str) -> PathBuf {
        self.path.join(name).join("salt")
    }

    fn secret_path(&self, name: &str) -> PathBuf {
        self.path.join(name).join("secret")
    }
}
