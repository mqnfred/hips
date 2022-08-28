use ::clishe::prelude::*;
use ::std::io::Write;

commands! {
    #[clap(help = "Store provided secret under the provided name")]
    Store(self, db: &mut hips::Database) -> Result<()> {
        db.store(hips::Secret{name: self.name, secret: self.secret})
    } struct {
        #[clap(help = "The name to store/hide the secret under")]
        name: String,
        #[clap(help = "The secret to store and hide")]
        secret: String,
    },

    #[clap(help = "Retrieve secret under the provided name")]
    Load(self, db: &mut hips::Database) -> Result<()> {
        writeln!(::std::io::stdout(), "{}", db.load(self.name)?.secret)?;
        Ok(())
    } struct {
        #[clap(help = "The name to retrieve the secrets for")]
        name: String,
    },

    #[clap(alias = "ls", help = "List all available secrets")]
    List(self, db: &mut hips::Database) -> Result<()> {
        let mut names = db.list()?.into_iter().map(|secret| {
            secret.name
        }).collect::<Vec<String>>();
        names.sort();
        writeln!(::std::io::stdout(), "{}", names.join("\n"))?;
        Ok(())
    } struct {},

    #[clap(alias = "rm", help = "Remove the secret under the provided name")]
    Remove(self, db: &mut hips::Database) -> Result<()> {
        db.remove(self.name)
    } struct {
        #[clap(help = "The name to retrieve the secrets for")]
        name: String,
    },

    #[clap(help = "Rename the secret to the provided name")]
    Rename(self, db: &mut hips::Database) -> Result<()> {
        let secret = db.load(self.current_name.clone())?;
        db.store(hips::Secret{name: self.new_name, secret: secret.secret})?;
        db.remove(self.current_name)
    } struct {
        #[clap(help = "Current name of the secret to move")]
        current_name: String,
        #[clap(help = "New name of the secret")]
        new_name: String,
    },

    #[clap(alias = "rot", help = "Re-encrypt the whole database using a new password")]
    Rotate(self, db: &mut hips::Database) -> Result<()> {
        let db_path = ::std::env::var("HIPS_DATABASE")?.into();
        let mut new_db = hips::Database::from_file(db_path, self.new_password)?;
        for secret in db.list()? {
            new_db.store(secret)?;
        }
        Ok(())
    } struct {
        #[clap(name = "new-password", help = "The password to re-encrypt the database with")]
        new_password: String,
    },

    #[clap(alias = "tmp", help = "Print one or multiple secrets according to a template")]
    Template(self, db: &mut hips::Database) -> Result<()> {
        let template = match ::std::fs::read_to_string(&self.template) {
            Err(err) if err.kind() == ::std::io::ErrorKind::NotFound => Ok(self.template),
            Err(err) => Err(err),
            Ok(val) => Ok(val),
        }?;
        writeln!(::std::io::stdout(), "{}", db.template(template)?)?;
        Ok(())
    } struct {
        #[clap(help = "Template or path to file containing the template")]
        template: String,
    },
}
