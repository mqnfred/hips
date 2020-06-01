#[macro_use]
extern crate clap;

use ::clap::Clap;
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
    Hips::parse().run(&mut ::hips::Database::new(db_path, password)?)
}

dispatchers! {
    #[clap(
        name = "hips",
        about = "Manage secrets alongside your code",
        author = "Louis Feuvrier <mqnfred@gmail.com>",
        version = "0.3.0",
        after_help = "\
            ENVIRONMENT:\n    \
            HIPS_DATABASE\tfile/folder containing the secrets (mandatory)\n    \
            HIPS_PASSWORD\tpassword that will unlock the database (mandatory)\
        ",
    )]
    Hips(self, _: &mut hips::Database) -> Result<()> [
        Store: commands::Store,
        Load: commands::Load,
        Remove: commands::Remove,
        Rotate: commands::Rotate,
        Template: commands::Template,
    ],
}
mod commands;

fn unwrap_env_var(name: &str) -> Result<String> {
    let var = ::std::env::var(name).map_err(|err| {
        Error::msg(format!("{}: {}", err, name))
    });

    if var.is_err() {
        eprintln!("hips expects both a database file/folder and its");
        eprintln!("password to be provided as environment variables");
    }

    var
}
