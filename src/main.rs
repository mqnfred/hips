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
    Env(env::Env),
    #[clap(name = "get", about = "Get the value for a given key")]
    Get(get::Get),
    #[clap(name = "set", about = "Set a key to a given value")]
    Set(set::Set),
}

mod env;
mod get;
mod set;
