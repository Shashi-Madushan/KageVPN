//! Turns pasted text, subscriptions and the share button into profiles and back, by
//! looking up the protocol that owns a link's scheme in the registry.

use super::{common::invalid, def, Profile, ProtocolId, REGISTRY};
use crate::error::Result;

/// The protocol whose scheme starts this token. Schemes are case-insensitive.
fn protocol_of(token: &str) -> Option<ProtocolId> {
    let (scheme, _) = token.split_once("://")?;
    let found = REGISTRY.iter().find(|d| d.schemes().iter().any(|s| s.eq_ignore_ascii_case(scheme)))?;
    ProtocolId::lookup(found.id())
}

fn is_link(token: &str) -> bool {
    protocol_of(token).is_some()
}

pub fn contains_link(text: &str) -> bool {
    text.split_whitespace().any(is_link)
}

pub fn parse_link(link: &str) -> Result<Profile> {
    let link = link.trim();
    let protocol = protocol_of(link).ok_or_else(|| invalid(format!("not a supported link: {}", supported_schemes())))?;
    def(protocol).parse(link)
}

/// Parses every supported link in a block of text, returning profiles and the errors.
pub fn parse_many(text: &str) -> (Vec<Profile>, Vec<String>) {
    let mut profiles = Vec::new();
    let mut errors = Vec::new();
    for token in text.split_whitespace().filter(|t| is_link(t)) {
        match parse_link(token) {
            Ok(profile) => profiles.push(profile),
            Err(error) => errors.push(error.to_string()),
        }
    }
    (profiles, errors)
}

pub fn to_link(profile: &Profile) -> String {
    def(profile.protocol).to_link(profile)
}

fn supported_schemes() -> String {
    REGISTRY.iter().flat_map(|d| d.schemes().iter()).map(|s| format!("{s}://")).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
#[path = "../tests/profile_links.rs"]
mod tests;
