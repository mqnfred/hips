#[macro_use]
extern crate serde;

use crate::prelude::*;
mod prelude {
    pub use crate::{Encrypted,Secret};
    pub use crate::{Backend,Database,Encrypter};
    pub use ::anyhow::{Context,Error,Result};
    pub use ::std::path::PathBuf;
    pub use ::std::io::{Read,Write};
}

pub struct Database {
    b: Box<dyn Backend>,
    e: Box<dyn Encrypter>,
}
mod database;

pub trait Backend {
    fn store(&mut self, encrypted: Encrypted) -> Result<()>;
    fn load(&mut self, name: String) -> Result<Encrypted>;
    fn remove(&mut self, name: String) -> Result<()>;
    fn list(&mut self) -> Result<Vec<Encrypted>>;
}
mod backends;

pub trait Encrypter {
    fn encrypt(&mut self, secret: Secret) -> Result<Encrypted>;
    fn decrypt(&mut self, encrypted: Encrypted) -> Result<Secret>;
}
mod encrypters;

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
