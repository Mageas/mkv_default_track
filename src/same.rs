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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = Same::new("en", "en-US", Some("English".to_owned()));
        assert_eq!(s.language, "en");
        assert_eq!(s.language_ietf, "en-US");
        assert_eq!(s.name, Some("English".to_owned()));
    }

    #[test]
    fn test_name_none() {
        let s = Same::new("fr", "fr-FR", None);
        assert_eq!(s.name, None);
    }

    #[test]
    fn test_language_diff() {
        let s = Same::new("fr", "fr-CA", Some("French".to_owned()));
        assert_ne!(s.language, s.language_ietf);
    }

    #[test]
    fn test_debug() {
        let s = Same::new("es", "es-ES", Some("Spanish".to_owned()));
        assert_eq!(
            format!("{:?}", s),
            "Same { language: \"es\", language_ietf: \"es-ES\", name: Some(\"Spanish\") }"
        );
    }

    #[test]
    fn test_eq() {
        let s1 = Same::new("de", "de-DE", Some("German".to_owned()));
        let s2 = Same::new("de", "de-DE", Some("German".to_owned()));
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_clone() {
        let s1 = Same::new("it", "it-IT", Some("Italian".to_owned()));
        let s2 = s1.clone();
        assert_eq!(s1, s2);
    }
}
