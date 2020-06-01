use ::std::iter::FromIterator;
use ::std::convert::TryFrom;
use crate::prelude::*;

impl Database {
    pub fn new(path: PathBuf, password: String) -> Result<Self> {
        if let Some(extension) = path.extension() {
            if extension == "yaml" {
                Ok(Database{
                    b: Box::new(crate::backends::YAML::new(path)),
                    e: Box::new(crate::encrypters::Ring::new(password)),
                })
            } else {
                Err(Error::msg(format!("unsupported format: {}", extension.to_str().unwrap())))
            }
        } else {
            Ok(Database{
                b: Box::new(crate::backends::Folder::new(path)),
                e: Box::new(crate::encrypters::Ring::new(password)),
            })
        }
    }
}

impl Database {
    pub fn store(&mut self, secret: Secret) -> Result<()> {
        let encrypted = self.e.encrypt(secret).context("encrypting secret")?;
        self.b.store(encrypted).context("storing secret")
    }

    pub fn load(&mut self, name: String) -> Result<Secret> {
        self.e.decrypt(
            self.b.load(name).context("looking up name")?
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

impl Database {
    pub fn template(&mut self, mut template: String) -> Result<String> {
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

impl ::std::convert::TryFrom<&mut Database> for TemplateContext {
    type Error = Error;
    fn try_from(db: &mut Database) -> Result<Self> {
        let secrets = db.list()?;
        Ok(Self{
            list: secrets.clone(),
            map: ::std::collections::HashMap::from_iter(
                secrets
                    .into_iter()
                    .map(|secret| (secret.name, secret.secret)),
            ),
        })
    }
}
