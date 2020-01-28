#[macro_use]
extern crate clap;

use ::anyhow::{Context,Error};
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

    let mut master = String::new();
    ::std::io::stdin().lock().read_to_string(&mut master)?;
    let db = ::hips::EncryptedYaml::new(opts.database, master);

    match opts.subcmd {
        Command::Get(get) => get.run(db),
        Command::Set(set) => set.run(db),
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
        pub fn run<S: ::hips::Store>(self, mut store: S) -> Result<(), Error> {
            Ok(writeln!(
                ::std::io::stdout(), "{}",
                store.get(self.name).context("retrieving secret")?,
            ).context("writing secret to stdout")?)
        }
    }

    #[derive(Clap, Debug)]
    pub struct Set {
        #[clap(name = "key")]
        key: String,
        #[clap(name = "value")]
        value: String,
    }
    impl Set {
        pub fn run<S: ::hips::Store>(self, mut store: S) -> Result<(), Error> {
            store.set(self.key, self.value).context("writing secret to database") 
        }
    }

    #[derive(Clap, Debug)]
    pub struct Env {
        #[clap(short = "i", long = "interpreter")]
        interpreter: Option<String>,
    }
    impl Env {
        pub fn run<S: ::hips::Store>(self, mut store: S) -> Result<(), Error> {
            let assignments = store.all().context("listing secrets")?.into_iter().map(|(k, v)| {
                format!("export {} = '{}';", k.to_uppercase(), v)
            }).collect::<Vec<String>>();

            if let Some(interpreter) = self.interpreter {
                writeln!(
                    ::std::io::stdout(),
                    "#!{}\n", interpreter,
                ).context("writing shebang to stdout")?;
            }
            Ok(writeln!(
                ::std::io::stdout(),
                "{}", assignments.join("\n")
            ).context("writing assignments to stdin")?)
        }
    }
}
