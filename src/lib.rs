//! A simple secrets database.
//!
//! While `hips` can be used as a binary for operational purposes, you might need your server to
//! retrieve secrets at runtime.
//!
//! You should be mostly manipulating the [`Database`][1] object directly. It exposes an API very
//! similar to the hips binary's (store, load, remove, list...)
//!
//! [1]: struct.Database.html

#[macro_use]
extern crate serde;

use crate::prelude::*;
mod prelude {
    pub use crate::{Backend, Database, Encrypter};
    pub use crate::{Encrypted, Secret};
    pub use anyhow::{Context, Error, Result};
    pub use std::io::{Read, Write};
    pub use std::path::PathBuf;
}

/// A handle to the underlying secrets database.
///
/// Any calls to this object's methods (load, store..) will result in a similar call on the
/// injected `Backend` implementation, no caching is being done. Every call will also invoke
/// encryption/decryption logic, which should be considered slow.
pub struct Database {
    b: Box<dyn Backend>,
    e: Box<dyn Encrypter>,
}
mod database;

/// Storage behavior: what does it mean to store/load/..?
///
/// A few backends are implemented by default ([`YAML`][1] and [`Folder`][2] at this time.) You are
/// free to implement your own `Backend` (that connects to a remote server for example) and
/// initialize a new `Database` with it.
///
/// [1]: backends/struct.YAML.html
/// [2]: backends/struct.Folder.html
pub trait Backend {
    fn store(&mut self, encrypted: Encrypted) -> Result<()>;
    fn load(&mut self, name: String) -> Result<Encrypted>;
    fn remove(&mut self, name: String) -> Result<()>;
    fn list(&mut self) -> Result<Vec<Encrypted>>;
}
pub mod backends;

/// Encryption behavior: what does it mean to encrypt/decrypt?
///
/// A single [`Ring`][1] encrypter is available at this time. In a past version, an openssl option
/// was also available. You are free to implement your own `Encrypter` and initialize a new
/// `Database` with it.
///
/// [1]: encrypters/struct.Ring.html
pub trait Encrypter {
    fn encrypt(&mut self, secret: Secret) -> Result<Encrypted>;
    fn decrypt(&mut self, encrypted: Encrypted) -> Result<Secret>;
}
pub mod encrypters;

/// A plaintext secret and its name.
///
/// Returned by the `decrypt` method of an [`Encrypter`][1] when provided an [`Encrypted`][2]
/// secret. Passing this secret to the same encrypter's `encrypt` method again might yield
/// different `Encrypted` data (this depends on the encrypter implementation.)
///
/// [1]: trait.Encrypter.html
/// [2]: struct.Encrypted.html
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Secret {
    pub name: String,
    pub secret: String,
}

/// An encrypted secret and its name.
///
/// Returned by the `encrypt` method of an [`Encrypter`][1] when provided a [`Secret`][2]. Should
/// yield the same `Secret` when passed to the same encrypter's `decrypt` method.
///
/// [1]: trait.Encrypter.html
/// [2]: struct.Secret.html
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encrypted {
    name: String,
    secret: String,
    salt: String,
}
