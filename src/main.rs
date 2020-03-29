#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde;

use ::anyhow::{Context,Error};
use ::hips::{Database,Secret};
use ::std::io::{Read,Write};
use ::std::iter::FromIterator;
use ::std::path::PathBuf;
use ::clap::Clap;

fn main() -> Result<(), Error> {
    if let Err(err) = run() {
        writeln!(::std::io::stderr(), "error: {:#}", err).context("error while printing error")?;
        ::std::process::exit(1);
    }
    Ok(())
}

fn run() -> Result<(), Error> {
    let opts = Options::parse();

    let mut password = String::new();
    let db_path: PathBuf = opts.database.into();
    ::std::io::stdin().lock().read_to_string(&mut password)?;
    let db = Database::new(db_path.clone(), password.clone())?;

    match opts.subcmd {
        Command::Set(set) => set.run(db),
        Command::Get(get) => get.run(db),
        Command::Delete(delete) => delete.run(db),
        Command::Rotate(rotate) => rotate.run(db, db_path, password),
        Command::Template(template) => template.run(db),
    }
}

#[derive(Clap, Debug)]
#[clap(version = "0.0.1", author = "Louis Feuvrier, mqnfred@gmail.com")]
struct Options {
    #[clap(short = "d", long = "db", help = "The secrets database to manipulate")]
    database: String,

    #[clap(subcommand)]
    subcmd: Command,
}

#[derive(Clap, Debug)]
enum Command {
    #[clap(name = "set", about = "Set a secret to a given value")]
    Set(commands::Set),
    #[clap(name = "get", about = "Get the value for a given secret name")]
    Get(commands::Get),
    #[clap(name = "delete", about = "Remove the given secret from the database")]
    Delete(commands::Delete),
    #[clap(name = "rotate", about = "Re-encrypt the whole database using a new password")]
    Rotate(commands::Rotate),
    #[clap(name = "template", about = "Print one or multiple secrets according to a template")]
    Template(commands::Template),
}

mod commands {
    use ::std::io::Write;
    use super::*;

    #[derive(Clap, Debug)]
    pub struct Set {
        #[clap(name = "name")]
        name: String,
        #[clap(name = "secret")]
        secret: String,
    }
    impl Set {
        pub fn run(self, mut db: Database) -> Result<(), Error> {
            db.set(
                Secret{name: self.name, secret: self.secret}
            ).context("writing secret to database") 
        }
    }

    #[derive(Clap, Debug)]
    pub struct Get {
        #[clap(name = "name")]
        name: String,
    }
    impl Get {
        pub fn run(self, mut db: Database) -> Result<(), Error> {
            Ok(writeln!(
                ::std::io::stdout(), "{}",
                db.get(self.name).context("retrieving secret")?.secret,
            ).context("writing secret to stdout")?)
        }
    }

    #[derive(Clap, Debug)]
    pub struct Delete {
        #[clap(name = "name")]
        name: String,
    }
    impl Delete {
        pub fn run(self, mut db: Database) -> Result<(), Error> {
            db.delete(self.name)
        }
    }

    #[derive(Clap, Debug)]
    pub struct Rotate {
        #[clap(long = "new-password")]
        new_password: Option<String>,
        #[clap(long = "new-path")]
        new_path: Option<PathBuf>,
    }
    impl Rotate {
        pub fn run(
            self,
            mut existing_db: Database,
            existing_path: PathBuf,
            existing_pw: String,
        ) -> Result<(), Error> {
            let path = self.new_path.unwrap_or(existing_path);
            let pw = self.new_password.unwrap_or(existing_pw);
            let mut new_db = Database::new(path.into(), pw).context("spinning up new db")?;
            for secret in existing_db.list().context("listing secrets")? {
                new_db.set(secret).context("adding secret to new db")?;
            }
            Ok(())
        }
    }

    #[derive(Clap, Debug)]
    pub struct Template {
        #[clap(name = "template", about = "File containing the template or template directly")]
        template: String,
    }
    impl Template {
        pub fn run(self, mut db: Database) -> Result<(), Error> {
            let mut template = match ::std::fs::read_to_string(&self.template) {
                Err(err) if err.kind() == ::std::io::ErrorKind::NotFound => Ok(self.template),
                Err(err) => Err(err),
                Ok(val) => Ok(val),
            }?;
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

            let secrets = db.list()?;
            let ctx = TemplateContext{
                list: secrets.clone(),
                map: ::std::collections::HashMap::from_iter(secrets.into_iter().map(|secret| {
                    (secret.name, secret.secret)
                })),
            };
            Ok(write!(::std::io::stdout(), "{}", tt.render("template", &ctx)?)?)
        }
    }
    #[derive(Serialize)]
    struct TemplateContext {
        list: Vec<Secret>,
        map: ::std::collections::HashMap<String, String>,
    }
}
