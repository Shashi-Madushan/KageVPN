//! Turns pasted text, subscriptions and the share button into profiles and back, by
//! looking up the protocol that owns a link's scheme in the registry.

use super::{common::invalid, def, formats, wireguard, Profile, ProtocolId, REGISTRY};
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

/// Link schemes people meet in subscriptions that this app cannot run yet. Naming them lets the
/// import say why servers were skipped instead of leaving a mystery.
const UNSUPPORTED: &[&str] = &["ssr", "socks", "socks4", "socks5", "anytls", "juicity", "mieru"];

fn unsupported_scheme(token: &str) -> Option<&'static str> {
    let (scheme, _) = token.split_once("://")?;
    UNSUPPORTED.iter().copied().find(|s| s.eq_ignore_ascii_case(scheme))
}

/// Some files and servers start with a byte order mark, which would hide the first link's scheme.
fn clean(text: &str) -> &str {
    text.trim_start_matches('\u{feff}')
}

pub fn contains_link(text: &str) -> bool {
    clean(text).split_whitespace().any(is_link)
}

pub fn parse_link(link: &str) -> Result<Profile> {
    let link = link.trim();
    let protocol = protocol_of(link).ok_or_else(|| invalid(format!("not a supported link: {}", supported_schemes())))?;
    def(protocol).parse(link)
}

/// Splits pasted text into links. They usually sit one per line, but a name after `#` may hold
/// unencoded spaces (`#My Server`) and some people paste several links on one line, so a link runs
/// until the next link starts. Words after a link that has no `#` are not part of it.
fn split_links(text: &str) -> Vec<String> {
    let mut links = Vec::new();
    for line in clean(text).lines() {
        let mut current: Option<String> = None;
        for token in line.split_whitespace() {
            if is_link(token) || unsupported_scheme(token).is_some() {
                links.extend(current.take());
                current = Some(token.to_string());
            } else if let Some(link) = current.as_mut().filter(|link| link.contains('#') && !token.contains("://")) {
                link.push(' ');
                link.push_str(token);
            }
        }
        links.extend(current);
    }
    links
}

/// What a block of text held: the servers, why entries were skipped (one per entry), and notes
/// about servers that were imported with something filled in.
#[derive(Default)]
pub struct Parsed {
    pub profiles: Vec<Profile>,
    pub errors: Vec<String>,
    pub notes: Vec<String>,
}

impl Parsed {
    fn add(&mut self, link: &str, result: Result<Profile>, label: Option<&str>) {
        match result {
            Ok(profile) => {
                if let Some(note) = def(profile.protocol).import_note(link) {
                    self.notes.push(format!("{}: {note}", profile.name));
                }
                self.profiles.push(profile);
            }
            Err(error) => self.errors.push(match label {
                Some(name) => format!("{name}: {error}"),
                None => error.to_string(),
            }),
        }
    }
}

/// Parses every supported link in a block of text, or every server in a sing-box or Clash
/// configuration file, or the one server of a WireGuard (wg-quick) file.
pub fn parse_all(text: &str) -> Parsed {
    parse_all_named(text, None)
}

/// `parse_all` for the text of a file: `file_name` names a WireGuard server read from it.
pub fn parse_all_named(text: &str, file_name: Option<&str>) -> Parsed {
    let mut parsed = Parsed::default();
    if wireguard::is_wg_quick(clean(text)) {
        let stem = file_name.map(|n| n.rsplit_once('.').filter(|(stem, _)| !stem.is_empty()).map_or(n, |(stem, _)| stem));
        match wireguard::parse_wg_quick(clean(text), stem) {
            Ok((profile, note)) => {
                parsed.notes.extend(note.map(|n| format!("{}: {n}", profile.name)));
                parsed.profiles.push(profile);
            }
            Err(error) => parsed.errors.push(error.to_string()),
        }
        return parsed;
    }
    if let Some(found) = formats::extract(text) {
        for entry in found {
            match entry {
                Ok((name, link)) => parsed.add(&link, parse_link(&link), Some(&name)),
                Err(reason) => parsed.errors.push(reason),
            }
        }
        return parsed;
    }
    for link in split_links(text) {
        if let Some(scheme) = unsupported_scheme(&link) {
            parsed.errors.push(format!("{scheme}:// links are not supported yet"));
            continue;
        }
        parsed.add(&link, parse_link(&link), None);
    }
    parsed
}

/// `parse_all` without the notes: profiles and the reasons any entry was skipped.
pub fn parse_many(text: &str) -> (Vec<Profile>, Vec<String>) {
    let parsed = parse_all(text);
    (parsed.profiles, parsed.errors)
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
