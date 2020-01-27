#[macro_use]
extern crate clap;

fn main() -> Result<(), ::failure::Error> {
    let opts = Options::parse();
    match opts.subcmd {
        Command::Env(env) => env.run(opts.store, opts.password),
        Command::Set(set) => set.run(opts.store, opts.password),
        Command::Get(get) => get.run(opts.store, opts.password),
        _ => panic!("please provide a command"),
    }
}

#[derive(Clap, Debug)]
#[clap(version = "0.0.1", author = "Louis Feuvrier, mqnfred@gmail.com")]
struct Options {
    #[clap(short = "s", long = "store")]
    store: String,
    #[clap(short = "p", long = "password")]
    password: String,

    #[clap(subcommand)]
    subcmd: Command,
}

#[derive(Clap, Debug)]
enum Command {
    #[clap(name = "env", about = "Output a shell script which loads all keys as environment")]
    Env(commands::Env),
    #[clap(name = "get", about = "Get the value for a given key")]
    Get(commands::Get),
    #[clap(name = "set", about = "Set a key to a given value")]
    Set(commands::Set),
}


mod commands {
    use ::hips::Store;
    use ::std::io::Write;

    #[derive(Clap, Debug)]
    pub struct Env {
        #[clap(short = "i", long = "interpreter")]
        interpreter: Option<String>,
    }
    impl Env {
        pub fn run(self, store: String, pw: String) -> Result<(), ::failure::Error> {
            let mut db = ::hips::EncryptedYaml::new(store, pw);
            let assignments = db.all()?.into_iter().map(|(k, v)| {
                format!("export {} = '{}';", k.to_uppercase(), v)
            }).collect::<Vec<String>>();

            if let Some(interpreter) = self.interpreter {
                writeln!(::std::io::stdout(), "#!{}\n", interpreter);
            }
            Ok(writeln!(::std::io::stdout(), "{}", assignments.join("\n"))?)
        }
    }

    #[derive(Clap, Debug)]
    pub struct Get {
        #[clap(name = "key")]
        key: String,
    }
    impl Get {
        pub fn run(self, store: String, pw: String) -> Result<(), ::failure::Error> {
            let mut db = ::hips::EncryptedYaml::new(store, pw);
            writeln!(::std::io::stdout(), "{}", db.get(self.key)?);
            Ok(())
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
        pub fn run(self, store: String, pw: String) -> Result<(), ::failure::Error> {
            let mut db = ::hips::EncryptedYaml::new(store, pw);
            db.set(self.key, self.value)
        }
    }
}
