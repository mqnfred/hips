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
        ::hips::YAMLStore::new(store, pw).set(self.key, self.value)
    }
}
