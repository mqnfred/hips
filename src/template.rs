use crate::prelude::*;

fn template(db: Database, template: String) -> String {
    
}

#[derive(Serialize)]
struct Context {
    list: Vec<Secret>,
    map: ::std::collections::HashMap<String, String>,
}

    #[derive(Clap, Debug)]
    pub struct Template {
        #[clap(
            name = "template",
            about = "File containing the template or template directly"
        )]
        template: String,
    }
    impl Template {
        pub fn run(self, mut db: Database) -> Result<()> {
            template = ::snailquote::unescape(&format!("\"{}\"", template))?;

            let mut tt = ::tinytemplate::TinyTemplate::new();
            tt.add_template("template", &template)?;
            tt.add_formatter("capitalize", |val, s| match val {
                ::serde_json::Value::String(string) => {
                    s.push_str(&string.to_uppercase());
                    Ok(())
                }
                _ => Err(Error::msg("can only capitalize strings")),
            });

            let secrets = db.list()?;
            let ctx = TemplateContext {
                list: secrets.clone(),
                map: ::std::collections::HashMap::from_iter(
                    secrets
                        .into_iter()
                        .map(|secret| (secret.name, secret.secret)),
                ),
            };
            Ok(write!(
                ::std::io::stdout(),
                "{}",
                tt.render("template", &ctx)?
            )?)
        }
    }
