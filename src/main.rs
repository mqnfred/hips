#[macro_use]
extern crate clap;

use ::clap::Parser;
use ::clishe::prelude::*;
use ::std::io::Write;

fn main() -> Result<()> {
    if let Err(err) = run() {
        writeln!(::std::io::stderr(), "error: {:#}", err)?;
        ::std::process::exit(1);
    }
    Ok(())
}

fn run() -> Result<()> {
    let db_path = unwrap_env_var("HIPS_DATABASE")?.into();
    let password = unwrap_env_var("HIPS_PASSWORD")?;
    Hips::parse().run(&mut ::hips::Database::from_file(db_path, password)?)
}

dispatchers! {
    #[clap(
        name = env!("CARGO_PKG_NAME"),
        about = env!("CARGO_PKG_DESCRIPTION"),
        author = env!("CARGO_PKG_AUTHORS"),
        version = env!("CARGO_PKG_VERSION"),
        after_help = "\
            ENVIRONMENT:\n    \
            HIPS_DATABASE    File/folder containing the secrets (mandatory)\n    \
            HIPS_PASSWORD    Password that will unlock the database (mandatory)\
        ",
    )]
    Hips(self, _: &mut hips::Database) -> Result<()> [
        Store: commands::Store,
        Load: commands::Load,
        List: commands::List,
        Remove: commands::Remove,
        Rename: commands::Rename,
        Rotate: commands::Rotate,
        Template: commands::Template,
    ],
}
mod commands;

fn unwrap_env_var(name: &str) -> Result<String> {
    let var = ::std::env::var(name).map_err(|err| Error::msg(format!("{}: {}", err, name)));

    if var.is_err() {
        eprintln!("hips expects both a database file/folder and its");
        eprintln!("password to be provided as environment variables");
    }

    var
}
