use takopack_core::{error::Error as CoreError, network::fetch_jsonl};

use crate::cargo::source::crates_io::{crates_io::FromCratesIo, error::Error};

pub struct FromCratesIoBuilder {
    name: String,
    version: Option<String>,
}

impl FromCratesIoBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            version: None,
        }
    }

    pub fn version(&mut self, version: String) -> &mut Self {
        self.version = Some(version);
        self
    }

    pub fn resolve(self) -> Result<FromCratesIo, Error> {
        let mut data_from_sprase_index = fetch_data_from_sprase_index(&self.name)?
            .into_iter()
            .filter(|info| !info.yanked);

        let crate_info = match self.version.as_deref() {
            Some(v) => data_from_sprase_index
                .find(|c: &CrateInfo| c.vers == v) // We believe crates.io gives no duplicated version.
                .ok_or_else(|| Error::VersionNotFound {
                    name: self.name.to_owned(),
                    version: v.to_owned(),
                }),
            // Assume that the last one crateinfo matches the latest version.
            // [TODO] Use SemVer to resolve the latest version.
            None => data_from_sprase_index
                .last()
                .ok_or_else(|| Error::CrateNotFound {
                    name: self.name.to_owned(),
                }),
        }?;

        Ok(FromCratesIo::new(crate_info))
    }
}

fn fetch_data_from_sprase_index(name: &str) -> Result<Vec<CrateInfo>, Error> {
    let url = sprase_index_url(name);

    match fetch_jsonl(&url) {
        Ok(v) => Ok(v),
        Err(CoreError::Network(ureq::Error::Status(404, _))) => Err(Error::CrateNotFound {
            name: name.to_owned(),
        }),
        Err(e) => Err(e.into()),
    }
}

fn sprase_index_url(name: &str) -> String {
    let len = name.len();
    let first_part = match len {
        0 => unreachable!(),
        1 => "1",
        2 => "2",
        3 => "3",
        _ => &name[0..=1],
    };
    let second_part = match len {
        0 => unreachable!(),
        1 => &name,
        2 => &name,
        3 => &name[0..=0],
        _ => &name[2..=3],
    };
    let third_part = match len {
        0 => unreachable!(),
        1 => None,
        2 => None,
        3 => Some(&name),
        _ => Some(&name),
    };

    let mut result = format!("https://index.crates.io/{}/{}", first_part, second_part);
    if let Some(third_part) = third_part {
        result.push('/');
        result.push_str(*third_part);
    }

    result
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct CrateInfo {
    pub name: String,
    pub vers: String,
    pub cksum: String,
    pub yanked: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprase_index_url() {
        assert_eq!(sprase_index_url("a"), "https://index.crates.io/1/a");
        assert_eq!(
            sprase_index_url("anyhow"),
            "https://index.crates.io/an/yh/anyhow"
        );
    }
}
