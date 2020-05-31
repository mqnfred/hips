#[macro_use]
extern crate clap;
#[macro_use]
extern crate modularcli;

use modularcli::prelude::*;
use ::std::io::Write;

fn main() -> Result<(), Error> {
    if let Err(err) = run() {
        writeln!(::std::io::stderr(), "error: {:#}", err)?;
        ::std::process::exit(1);
    }
    Ok(())
}

fn run() -> Result<(), Error> {
    let db_path = ::std::env::var("HIPS_DATABASE")?.into();
    let password = ::std::env::var("HIPS_PASSWORD")?;
    Hips::parse().run(&mut ::hips::Database::new(db_path, password)?)
}

dispatchers! {
    #[clap(author = "Louis Feuvrier, mqnfred@gmail.com")]
    Hips(self, _: &mut hips::Database) -> Result<()> [
        Store: commands::Store,
        Load: commands::Load,
        Remove: commands::Remove,
        Rotate: commands::Rotate,
        Template: commands::Template,
    ],
}

mod commands {
    use ::modularcli::prelude::*;
    use ::std::io::Write;

    commands! {
        #[clap(about = "Store provided secret under the provided name")]
        Store(self, db: &mut hips::Database) -> Result<()> {
            db.store(hips::Secret{name: self.name, secret: self.secret})
        } struct {
            #[clap(about = "The name to store/hide the secret under")]
            name: String,
            #[clap(about = "The secret to store and hide")]
            secret: String,
        },

        #[clap(about = "Retrieve secret under the provided name")]
        Load(self, db: &mut hips::Database) -> Result<()> {
            writeln!(::std::io::stdout(), "{}", db.load(self.name)?.secret)?;
            Ok(())
        } struct {
            #[clap(about = "The name to retrieve the secrets for")]
            name: String,
        },

        #[clap(alias = "rm", about = "Remove the secret under the provided name")]
        Remove(self, db: &mut hips::Database) -> Result<()> {
            db.remove(self.name)
        } struct {
            #[clap(about = "The name to retrieve the secrets for")]
            name: String,
        },

        #[clap(alias = "rot", about = "Re-encrypt the whole database using a new password")]
        Rotate(self, db: &mut hips::Database) -> Result<()> {
            let db_path = ::std::env::var("HIPS_DATABASE")?.into();
            let mut new_db = hips::Database::new(db_path, self.new_password)?;
            for secret in db.list()? {
                new_db.store(secret)?;
            }
            Ok(())
        } struct {
            #[clap(name = "new-password", about = "The password to re-encrypt the database with")]
            new_password: String,
        },

        #[clap(alias = "tmp", about = "Print one or multiple secrets according to a template")]
        Template(self, db: &mut hips::Database) -> Result<()> {
            let template = match ::std::fs::read_to_string(&self.template) {
                Err(err) if err.kind() == ::std::io::ErrorKind::NotFound => Ok(self.template),
                Err(err) => Err(err),
                Ok(val) => Ok(val),
            }?;
            writeln!(::std::io::stdout(), "{}", db.template(template)?)?;
            Ok(())
        } struct {
            #[clap(about = "Template or path to file containing the template")]
            template: String,
        },
    }
}
