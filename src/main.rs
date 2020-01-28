#[macro_use]
extern crate clap;

use ::anyhow::{Context,Error};
use ::hips::{Database,Backend,Encrypter,Secret};
use ::std::io::{Read,Write};

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
    ::std::io::stdin().lock().read_to_string(&mut password)?;
    let db = Database::<::hips::YAML, ::hips::OpenSSL>::new(opts.database.clone(), password);

    match opts.subcmd {
        Command::Get(get) => get.run(db),
        Command::Set(set) => set.run(db),
        Command::Rot(rot) => rot.run(db, opts.database),
        Command::Env(env) => env.run(db),
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
    #[clap(name = "get", about = "Get the value for a given secret name")]
    Get(commands::Get),
    #[clap(name = "set", about = "Set a secret to a given value")]
    Set(commands::Set),
    #[clap(name = "rot", about = "Re-encrypt the whole database using a new password")]
    Rot(commands::Rot),
    #[clap(name = "env", about = "Output scripts which load all secrets as environment variables")]
    Env(commands::Env),
}

mod commands {
    use ::std::io::Write;
    use super::*;

    #[derive(Clap, Debug)]
    pub struct Get {
        #[clap(name = "name")]
        name: String,
    }
    impl Get {
        pub fn run<B: Backend, E: Encrypter>(self, mut db: Database<B, E>) -> Result<(), Error> {
            Ok(writeln!(
                ::std::io::stdout(), "{}",
                db.get(self.name).context("retrieving secret")?.secret,
            ).context("writing secret to stdout")?)
        }
    }

    #[derive(Clap, Debug)]
    pub struct Set {
        #[clap(name = "name")]
        name: String,
        #[clap(name = "secret")]
        secret: String,
    }
    impl Set {
        pub fn run<B: Backend, E: Encrypter>(self, mut db: Database<B, E>) -> Result<(), Error> {
            db.set(
                Secret{name: self.name, secret: self.secret}
            ).context("writing secret to database") 
        }
    }

    #[derive(Clap, Debug)]
    pub struct Rot {
        #[clap(name = "new-password")]
        new_password: String,
    }
    impl Rot {
        pub fn run<B: Backend, E: Encrypter>(
            self,
            mut db: Database<B, E>,
            db_path: String,
        ) -> Result<(), Error> {
            let mut new_db = Database::<::hips::YAML, ::hips::OpenSSL>::new(db_path, self.new_password);
            for secret in db.all().context("listing secrets")? {
                new_db.set(secret).context("adding secret to new db")?;
            }
            Ok(())
        }
    }

    #[derive(Clap, Debug)]
    pub struct Env {
        #[clap(long = "shell")]
        shell: Option<String>,
    }
    impl Env {
        pub fn run<B: Backend, E: Encrypter>(self, mut db: Database<B, E>) -> Result<(), Error> {
            let assignments = db.all().context("listing secrets")?.into_iter().map(|s| {
                format!("export {}='{}';", s.name.to_uppercase(), s.secret)
            }).collect::<Vec<String>>();

            if let Some(shell) = self.shell {
                writeln!(::std::io::stdout(), "#!{}", shell).context("writing shebang to stdout")?;
            }
            Ok(writeln!(
                ::std::io::stdout(),
                "{}", assignments.join("\n")
            ).context("writing assignments to stdin")?)
        }
    }
}
