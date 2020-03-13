#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde;

use ::anyhow::{Context,Error};
use ::hips::{Database,Secret};
use ::std::io::{Read,Write};
use ::std::path::PathBuf;

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
        Command::Get(get) => get.run(db),
        Command::Set(set) => set.run(db),
        Command::Rot(rot) => rot.run(db, db_path, password),
        Command::All(all) => all.run(db),
        Command::Del(del) => del.run(db),
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
    #[clap(name = "del", about = "Remove the given secret from the database")]
    Del(commands::Del),
    #[clap(name = "rot", about = "Re-encrypt the whole database using a new password")]
    Rot(commands::Rot),
    #[clap(name = "all", about = "Print all initializations, provide template")]
    All(commands::All),
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
    pub struct Del {
        #[clap(name = "name")]
        name: String,
    }
    impl Del {
        pub fn run(self, mut db: Database) -> Result<(), Error> {
            db.del(self.name)
        }
    }

    #[derive(Clap, Debug)]
    pub struct Rot {
        #[clap(long = "new-password")]
        new_password: Option<String>,
        #[clap(long = "new-path")]
        new_path: Option<PathBuf>,
    }
    impl Rot {
        pub fn run(
            self,
            mut existing_db: Database,
            existing_path: PathBuf,
            existing_pw: String,
        ) -> Result<(), Error> {
            let path = self.new_path.unwrap_or(existing_path);
            let pw = self.new_password.unwrap_or(existing_pw);
            let mut new_db = Database::new(path.into(), pw).context("spinning up new db")?;
            for secret in existing_db.all().context("listing secrets")? {
                new_db.set(secret).context("adding secret to new db")?;
            }
            Ok(())
        }
    }

    #[derive(Clap, Debug)]
    pub struct All {
        #[clap(
            short = "t",
            long = "template",
            about = "Jinja-style template. You can iterate over secrets as such:\n\
                {{ for secret in secrets }}\n\
                {secret.name|capitalize}={secret.secret}\n\
                {{ endfor }}\n\
                For more features, see `tinytemplate` rust crate.\n\
                \n\
                If the template given evaluates to a file, the\n\
                template will be read from that file.",
        )]
        template: String,
    }
    impl All {
        pub fn run(self, mut db: Database) -> Result<(), Error> {
            let mut template = match ::std::fs::read_to_string(&self.template) {
                Err(err) if err.kind() == ::std::io::ErrorKind::NotFound => Ok(self.template),
                Err(err) => Err(err),
                Ok(val) => Ok(val),
            }?;
            template = ::snailquote::unescape(&format!("\"{}\"", template))?;

            let mut tt = ::tinytemplate::TinyTemplate::new();
            tt.add_template("all", &template)?;
            tt.add_formatter("capitalize", |val, s| match val {
                ::serde_json::Value::String(string) => {
                    s.push_str(&string.to_uppercase());
                    Ok(())
                }
                _ => panic!("can only capitalize strings"),
            });

            let ctx = AllContext{secrets: db.all()?};
            Ok(write!(::std::io::stdout(), "{}", tt.render("all", &ctx)?)?)
        }
    }
    #[derive(Serialize)]
    struct AllContext {
        secrets: Vec<Secret>,
    }
}
