use crate::prelude::*;
use ::std::convert::TryFrom;
use ::std::iter::FromIterator;

impl Database {
    /// Instantiate a new `Database` with injected `Backend`/`Encrypter`.
    pub fn new(backend: Box<dyn crate::Backend>, encrypter: Box<dyn crate::Encrypter>) -> Self {
        Self {
            b: backend,
            e: encrypter,
        }
    }

    /// Instantiate a new `Database` from a file.
    ///
    /// This function will try to guess which database type is needed, and create the appropriate
    /// backend based on that. This is used in the binary, which supports only the `Backend`s and
    /// `Encrypter`s shipped with hips.
    pub fn from_file(path: PathBuf, password: String) -> Result<Self> {
        if let Some(extension) = path.extension() {
            if extension == "yaml" {
                Ok(Self::new(
                    Box::new(crate::backends::YAML::new(path)),
                    Box::new(crate::encrypters::Ring::new(password)),
                ))
            } else {
                Err(Error::msg(format!(
                    "unsupported format: {}",
                    extension.to_str().unwrap()
                )))
            }
        } else {
            Ok(Self::new(
                Box::new(crate::backends::Folder::new(path)),
                Box::new(crate::encrypters::Ring::new(password)),
            ))
        }
    }
}

impl Database {
    /// Store the provided secret.
    pub fn store(&mut self, secret: Secret) -> Result<()> {
        let encrypted = self.e.encrypt(secret).context("encrypting secret")?;
        self.b.store(encrypted).context("storing secret")
    }

    /// Load the `name` secret.
    pub fn load(&self, name: String) -> Result<Secret> {
        self.e
            .decrypt(self.b.load(name).context("looking up name")?)
            .context("decrypting secret")
    }

    /// Remove the `name` secret.
    pub fn remove(&mut self, name: String) -> Result<()> {
        self.b.remove(name).context("removing secret")
    }

    /// List all secrets.
    pub fn list(&self) -> Result<Vec<Secret>> {
        self.b
            .list()
            .context("listing secrets")?
            .into_iter()
            .map(|s| self.e.decrypt(s))
            .collect::<Result<Vec<Secret>>>()
    }
}

impl Database {
    /// Process the database through a template.
    ///
    /// This call will read the provided `template` and replace all references to secrets with the
    /// secrets stored in the database. We use the [tinytemplate][1] engine, see their [syntax
    /// page][2] for more context.
    ///
    /// [1]: https://crates.io/crates/tinytemplate
    /// [2]: https://docs.rs/tinytemplate/1.0.4/tinytemplate/syntax/index.html
    pub fn template(&self, mut template: String) -> Result<String> {
        template = ::snailquote::unescape(&format!("\"{}\"", template))?;

        let mut tt = ::tinytemplate::TinyTemplate::new();
        tt.add_template("template", &template)?;
        tt.add_formatter("capitalize", |val, s| match val {
            ::serde_json::Value::String(string) => {
                s.push_str(&string.to_uppercase());
                Ok(())
            }
            _ => panic!("can only capitalize strings"),
        });

        let tctx = TemplateContext::try_from(self)?;
        Ok(tt.render("template", &tctx)?)
    }
}

#[derive(Serialize)]
struct TemplateContext {
    list: Vec<Secret>,
    map: ::std::collections::HashMap<String, String>,
}

impl ::std::convert::TryFrom<&Database> for TemplateContext {
    type Error = Error;
    fn try_from(db: &Database) -> Result<Self> {
        let secrets = db.list()?;
        Ok(Self {
            list: secrets.clone(),
            map: ::std::collections::HashMap::from_iter(
                secrets
                    .into_iter()
                    .map(|secret| (secret.name, secret.secret)),
            ),
        })
    }
}
