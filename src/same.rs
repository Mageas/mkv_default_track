#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Same {
    pub language: String,
    pub language_ietf: String,
    pub name: Option<String>,
}

impl Same {
    pub fn new(language: &str, language_ietf: &str, name: Option<String>) -> Self {
        Self {
            language: language.to_owned(),
            language_ietf: language_ietf.to_owned(),
            name,
        }
    }
}
