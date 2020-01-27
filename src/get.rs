use ::hips::Store;
use ::std::io::Write;

#[derive(Clap, Debug)]
pub struct Get {
    #[clap(name = "key")]
    key: String,
}

impl Get {
    pub fn run(self, store: String, pw: String) -> Result<(), ::failure::Error> {
        writeln!(::std::io::stdout(), "{}", ::hips::YAMLStore::new(store, pw).get(self.key)?);
        Ok(())
    }
}
