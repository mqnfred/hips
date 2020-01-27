use ::hips::Store;

#[derive(Clap, Debug)]
pub struct Set {
    #[clap(name = "key")]
    key: String,
    #[clap(name = "value")]
    value: String,
}

impl Set {
    pub fn run(self, store: String, pw: String) -> Result<(), ::failure::Error> {
        let mut store = ::hips::YAMLStore::<::hips::MagicEncrypter>::new(store, pw);
        store.set(self.key, self.value)
    }
}
