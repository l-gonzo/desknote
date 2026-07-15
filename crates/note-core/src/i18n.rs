use std::collections::HashMap;

const ES_MX: &str = include_str!("../../../locales/es-MX/shell.ftl");
const EN_US: &str = include_str!("../../../locales/en-US/shell.ftl");
const PT_BR: &str = include_str!("../../../locales/pt-BR/shell.ftl");
const FR_FR: &str = include_str!("../../../locales/fr-FR/shell.ftl");
const DE_DE: &str = include_str!("../../../locales/de-DE/shell.ftl");

#[derive(Debug, Clone)]
pub struct Translator {
    locale: String,
    values: HashMap<String, String>,
    fallback: HashMap<String, String>,
}

impl Translator {
    pub fn new(locale: impl Into<String>) -> Self {
        let locale = locale.into();
        let source = match locale.as_str() {
            "en-US" | "en" => EN_US,
            "pt-BR" | "pt" => PT_BR,
            "fr-FR" | "fr" => FR_FR,
            "de-DE" | "de" => DE_DE,
            _ => ES_MX,
        };
        Self {
            locale,
            values: parse(source),
            fallback: parse(EN_US),
        }
    }

    pub fn locale(&self) -> &str {
        &self.locale
    }

    pub fn text(&self, key: &str) -> String {
        self.values
            .get(key)
            .or_else(|| self.fallback.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_owned())
    }

    pub fn format(&self, key: &str, replacements: &[(&str, &str)]) -> String {
        let mut value = self.text(key);
        for (name, replacement) in replacements {
            value = value.replace(&format!("{{ ${name} }}"), replacement);
        }
        value
    }
}

fn parse(source: &str) -> HashMap<String, String> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
                return None;
            }
            let (key, value) = trimmed.split_once('=')?;
            Some((key.trim().to_owned(), value.trim().to_owned()))
        })
        .collect()
}
