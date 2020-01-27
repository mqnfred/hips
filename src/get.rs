use ::hips::Store;
use ::std::io::Write;

#[derive(Clap, Debug)]
pub struct Get {
    #[clap(name = "key")]
    key: String,
}

impl Get {
    pub fn run(self, store: String, pw: String) -> Result<(), ::failure::Error> {
        let mut store = ::hips::YAMLStore::<::hips::MagicEncrypter>::new(store, pw);
        writeln!(::std::io::stdout(), "{}", store.get(self.key)?);
        Ok(())
    }
}
