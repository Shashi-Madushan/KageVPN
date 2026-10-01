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
        Some(Self { numbers, pre })
    }
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
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).unwrap()
    }

    #[test]
    fn parses_with_and_without_prefix() {
        assert_eq!(v("v1.14.2"), v("1.14.2"));
        assert!(Version::parse("1.14").is_none());
        assert!(Version::parse("abc").is_none());
    }

    #[test]
    fn orders_numerically_not_lexically() {
        assert!(v("1.14.2") > v("1.9.10"));
        assert!(v("1.15.0") > v("1.14.9"));
    }

    #[test]
    fn prerelease_sorts_before_its_release() {
        assert!(v("1.15.0-alpha.9") < v("1.15.0"));
        assert!(v("1.15.0-alpha.10") > v("1.15.0-alpha.9"));
        assert!(v("1.15.0-alpha.1") > v("1.14.2"));
    }
}
