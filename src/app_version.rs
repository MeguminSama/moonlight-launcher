//! Helpers for ordering Discord `app-<version>` installation folders.

/// Compare two Discord app directory names (e.g. `app-1.2.3`, `app-2.0.0`)
/// such that higher versions sort first.
///
/// Entries whose names cannot be parsed as `app-<version>` sort after all
/// valid ones, keeping the original order among themselves.
#[cfg(any(windows, test))]
pub(crate) fn compare_app_dirs(a: &str, b: &str) -> std::cmp::Ordering {
    fn parse_version(s: &str) -> Result<Vec<u32>, ()> {
        // Split into prefix and version parts
        let version_str = s.split_once('-').map(|x| x.1).ok_or(())?;
        // Parse each numeric component
        version_str
            .split('.')
            .map(|num| num.parse().map_err(|_| ()))
            .collect()
    }

    match (parse_version(a), parse_version(b)) {
        (Ok(a_ver), Ok(b_ver)) => b_ver.cmp(&a_ver), // Both valid: compare versions
        (Ok(_), Err(_)) => std::cmp::Ordering::Less, // Valid < Invalid
        (Err(_), Ok(_)) => std::cmp::Ordering::Greater, // Invalid > Valid
        (Err(_), Err(_)) => std::cmp::Ordering::Equal, // Invalid entries stay at the end
    }
}

#[cfg(test)]
mod tests {
    use super::compare_app_dirs;
    use std::cmp::Ordering;

    fn sorted<'a>(dirs: &[&'a str]) -> Vec<&'a str> {
        let mut dirs = dirs.to_vec();
        dirs.sort_by(|a, b| compare_app_dirs(a, b));
        dirs
    }

    #[test]
    fn sorts_highest_version_first() {
        assert_eq!(
            sorted(&["app-1.2.3", "app-2.0.0", "app-1.9.9"]),
            vec!["app-2.0.0", "app-1.9.9", "app-1.2.3"]
        );
    }

    #[test]
    fn compares_numeric_componentwise() {
        // 10 > 9 only if components are parsed as numbers, not strings.
        assert_eq!(
            sorted(&["app-9.0.0", "app-10.0.0"]),
            vec!["app-10.0.0", "app-9.0.0"]
        );
    }

    #[test]
    fn shorter_version_sorts_after_same_prefix() {
        assert_eq!(sorted(&["app-1.2", "app-1.2.3"]), vec!["app-1.2.3", "app-1.2"]);
    }

    #[test]
    fn invalid_names_sort_last() {
        assert_eq!(
            sorted(&["app-1.2.3", "garbage", "app-x.y.z", "app-1.0.0"]),
            vec!["app-1.2.3", "app-1.0.0", "garbage", "app-x.y.z"]
        );
    }

    #[test]
    fn equal_versions_are_equal() {
        assert_eq!(compare_app_dirs("app-1.2.3", "app-1.2.3"), Ordering::Equal);
    }
}
