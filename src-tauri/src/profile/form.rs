//! How a protocol's settings are shown in the editor. Each protocol lists its own fields and
//! the interface renders them, so adding a protocol needs no interface code.

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Text,
    /// A credential; shown as typed, in a monospace face.
    Secret,
    Select,
    Segmented,
    Toggle,
    /// Comma-separated values stored as a list.
    List,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Section {
    Connection,
    Security,
    Transport,
    Advanced,
}

#[derive(Clone, Debug, Serialize)]
pub struct Choice {
    pub value: String,
    pub label: String,
}

/// Show a field only while another field holds one of these values.
#[derive(Clone, Debug, Serialize)]
pub struct ShowWhen {
    pub key: &'static str,
    pub values: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Pattern {
    /// A JavaScript regular expression, matched case-insensitively.
    pub regex: &'static str,
    pub message: &'static str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    /// Where the value lives in the profile, such as `sni` or `transport.path`.
    pub key: &'static str,
    pub label: &'static str,
    pub kind: Kind,
    pub section: Section,
    pub choices: Vec<Choice>,
    pub optional: bool,
    pub mono: bool,
    pub placeholder: Option<&'static str>,
    /// Shown under the field while it is switched on or filled in.
    pub warning: Option<&'static str>,
    pub show_when: Option<ShowWhen>,
    pub pattern: Option<Pattern>,
}

impl Field {
    fn new(kind: Kind, key: &'static str, label: &'static str, section: Section) -> Self {
        Self { key, label, kind, section, choices: Vec::new(), optional: false, mono: false, placeholder: None, warning: None, show_when: None, pattern: None }
    }

    pub fn text(key: &'static str, label: &'static str, section: Section) -> Self {
        Self::new(Kind::Text, key, label, section)
    }

    pub fn secret(key: &'static str, label: &'static str, section: Section) -> Self {
        Self { mono: true, ..Self::new(Kind::Secret, key, label, section) }
    }

    pub fn toggle(key: &'static str, label: &'static str, section: Section) -> Self {
        Self::new(Kind::Toggle, key, label, section)
    }

    pub fn list(key: &'static str, label: &'static str, section: Section) -> Self {
        Self { mono: true, ..Self::new(Kind::List, key, label, section) }
    }

    pub fn select(key: &'static str, label: &'static str, section: Section, choices: &[(&str, &str)]) -> Self {
        Self { choices: choices_of(choices), ..Self::new(Kind::Select, key, label, section) }
    }

    pub fn segmented(key: &'static str, label: &'static str, section: Section, choices: &[(&str, &str)]) -> Self {
        Self { choices: choices_of(choices), ..Self::new(Kind::Segmented, key, label, section) }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }

    pub fn placeholder(mut self, text: &'static str) -> Self {
        self.placeholder = Some(text);
        self
    }

    pub fn warning(mut self, text: &'static str) -> Self {
        self.warning = Some(text);
        self
    }

    pub fn show_when(mut self, key: &'static str, values: &[&'static str]) -> Self {
        self.show_when = Some(ShowWhen { key, values: values.to_vec() });
        self
    }

    pub fn pattern(mut self, regex: &'static str, message: &'static str) -> Self {
        self.pattern = Some(Pattern { regex, message });
        self
    }
}

fn choices_of(choices: &[(&str, &str)]) -> Vec<Choice> {
    choices.iter().map(|(value, label)| Choice { value: value.to_string(), label: label.to_string() }).collect()
}

/// What the interface needs to list a protocol and draw its editor.
#[derive(Serialize)]
pub struct ProtocolView {
    pub id: &'static str,
    pub label: &'static str,
    pub schemes: &'static [&'static str],
    pub fields: Vec<Field>,
}

#[cfg(test)]
#[path = "../tests/profile_form.rs"]
mod tests;
