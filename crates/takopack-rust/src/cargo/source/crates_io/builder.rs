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
            // The sparse index is append-only and not necessarily sorted by
            // version, so resolve the latest version by comparing semver.
            None => select_latest(data_from_sprase_index).ok_or_else(|| Error::CrateNotFound {
                name: self.name.to_owned(),
            }),
        }?;

        Ok(FromCratesIo::new(crate_info))
    }
}

/// Select the newest of `candidates` by semver.
///
/// Stable releases take precedence over pre-releases, so a pre-release is only
/// returned when no stable release is available. Entries whose `vers` is not a
/// valid semver are ignored; crates.io should never publish such versions.
fn select_latest(candidates: impl Iterator<Item = CrateInfo>) -> Option<CrateInfo> {
    let mut latest_stable: Option<(semver::Version, CrateInfo)> = None;
    let mut latest_prerelease: Option<(semver::Version, CrateInfo)> = None;

    for info in candidates {
        let Ok(version) = semver::Version::parse(&info.vers) else {
            continue;
        };

        let slot = if version.pre.is_empty() {
            &mut latest_stable
        } else {
            &mut latest_prerelease
        };

        if slot.as_ref().is_none_or(|(current, _)| version > *current) {
            *slot = Some((version, info));
        }
    }

    latest_stable.or(latest_prerelease).map(|(_, info)| info)
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

    fn crate_info(vers: &str) -> CrateInfo {
        CrateInfo {
            name: "test".to_owned(),
            vers: vers.to_owned(),
            cksum: "test-checksum".to_owned(),
            yanked: false,
        }
    }

    #[test]
    fn test_select_latest_uses_semver_order() {
        // Publishing order does not match semver order.
        let candidates = ["0.9.0", "0.10.0", "0.2.0"].into_iter().map(crate_info);

        assert_eq!(select_latest(candidates).unwrap().vers, "0.10.0");
    }

    #[test]
    fn test_select_latest_prefers_stable_over_prerelease() {
        let candidates = ["1.0.0", "2.0.0-rc.1"].into_iter().map(crate_info);

        assert_eq!(select_latest(candidates).unwrap().vers, "1.0.0");
    }

    #[test]
    fn test_select_latest_falls_back_to_prerelease() {
        let candidates = ["1.0.0-alpha.1", "1.0.0-beta.1"]
            .into_iter()
            .map(crate_info);

        assert_eq!(select_latest(candidates).unwrap().vers, "1.0.0-beta.1");
    }

    #[test]
    fn test_select_latest_skips_invalid_versions() {
        let candidates = ["not-a-version", "1.2.3"].into_iter().map(crate_info);

        assert_eq!(select_latest(candidates).unwrap().vers, "1.2.3");
    }

    #[test]
    fn test_select_latest_with_no_candidates() {
        assert!(select_latest(std::iter::empty()).is_none());
    }
}
