use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use toml;

type Target = PathBuf;
type Link = PathBuf;
type Records<K, V> = BTreeMap<K, V>;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct DotConfig {
    records: Records<Target, Link>,
}

pub fn test_config() {
    let config = DotConfig {
        records: Records::from([
            (PathBuf::from("A"), PathBuf::from("a")),
            (PathBuf::from("~/Applications/df as/s"), PathBuf::from("a")),
        ]),
    };

    println!("{}", toml::to_string(&config).unwrap())
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_serialize() {
        let config = DotConfig {
            records: Records::from([
                (PathBuf::from("B"), PathBuf::from("~/a/b/c")),
                (PathBuf::from("A"), PathBuf::from("a")),
                (PathBuf::from("a/b"), PathBuf::from("~/a/b/")),
            ]),
        };
        let expected_str = r#"[records]
A = "a"
B = "~/a/b/c"
"a/b" = "~/a/b/"
"#;
        let actual = toml::to_string(&config).unwrap();
        println!("{}", actual);
        assert_eq!(actual, expected_str);
    }

    #[test]
    fn test_deserialize() {
        let toml_content = r#"[records]
A = "a"
B = "~/a/b/c"
"a/b" = "~/a/b"
"#;
        let expected_config = DotConfig {
            records: Records::from([
                (PathBuf::from("B"), PathBuf::from("~/a/b/c")),
                (PathBuf::from("A"), PathBuf::from("a")),
                (PathBuf::from("a/b"), PathBuf::from("~/a/b/")),
            ]),
        };
        let actual: DotConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(actual, expected_config);
    }
}
