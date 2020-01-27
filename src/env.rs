use ::hips::Store;
use ::std::io::Write;

#[derive(Clap, Debug)]
pub struct Env {
    #[clap(short = "i", long = "interpreter")]
    interpreter: Option<String>,
}

impl Env {
    pub fn run(self, store: String, pw: String) -> Result<(), ::failure::Error> {
        let assignments = ::hips::YAMLStore::new(store, pw).all()?.into_iter().map(|(k, v)| {
            format!("export {} = '{}';", k.to_uppercase(), v)
        }).collect::<Vec<String>>();

        if let Some(interpreter) = self.interpreter {
            writeln!(::std::io::stdout(), "#!{}\n", interpreter);
        }

        Ok(writeln!(::std::io::stdout(), "{}", assignments.join("\n"))?)
    }
}
