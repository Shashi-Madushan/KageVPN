use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    numbers: [u32; 3],
    pre: Option<String>,
}

impl Version {
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim().trim_start_matches('v');
        let (core, pre) = match text.split_once('-') {
            Some((core, pre)) => (core, Some(pre.to_string())),
            None => (text, None),
        };
        let mut parts = core.split('.').map(|p| p.parse::<u32>().ok());
        let numbers = [parts.next()??, parts.next()??, parts.next()??];
        // A version names a folder, so it cannot carry path separators or climb out with `..`.
        if parts.next().is_some() || pre.as_deref().is_some_and(|pre| !is_plain_prerelease(pre)) {
            return None;
        }
        Some(Self { numbers, pre })
    }
}

fn is_plain_prerelease(pre: &str) -> bool {
    !pre.is_empty() && !pre.contains("..") && pre.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.numbers.cmp(&other.numbers).then_with(|| match (&self.pre, &other.pre) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => compare_identifiers(a, b),
        })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn compare_identifiers(a: &str, b: &str) -> Ordering {
    let (mut left, mut right) = (a.split('.'), b.split('.'));
    loop {
        match (left.next(), right.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let order = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(x), Ok(y)) => x.cmp(&y),
                    _ => x.cmp(y),
                };
                if order != Ordering::Equal {
                    return order;
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/core_version.rs"]
mod tests;
