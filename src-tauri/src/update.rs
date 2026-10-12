use semver::Version;
use serde::Deserialize;

use crate::git::types::UpdateInfo;

pub use crate::git::types::UpdateCheck;

#[derive(Debug, Deserialize)]
pub struct ReleaseEntry {
    pub tag_name: String,
    pub html_url: String,
    pub draft: bool,
    pub prerelease: bool,
}

fn parsed_version(tag: &str) -> Option<Version> {
    Version::parse(tag.trim_start_matches('v')).ok()
}

fn info(release: &ReleaseEntry, version: &Version) -> UpdateInfo {
    UpdateInfo {
        version: version.to_string(),
        release_url: release.html_url.clone(),
    }
}

pub fn pick_update(releases: &[ReleaseEntry], current: &Version) -> UpdateCheck {
    let mut same_major: Option<(&ReleaseEntry, Version)> = None;
    let mut next_major: Option<(&ReleaseEntry, Version)> = None;

    for release in releases {
        if release.draft || release.html_url.is_empty() {
            continue;
        }
        let Some(version) = parsed_version(&release.tag_name) else {
            continue;
        };

        if version.major == current.major
            && version > *current
            && (!current.pre.is_empty() || !release.prerelease)
            && same_major.as_ref().is_none_or(|(_, best)| version > *best)
        {
            same_major = Some((release, version.clone()));
        }

        if version.major == current.major.saturating_add(1)
            && !release.prerelease
            && next_major.as_ref().is_none_or(|(_, best)| version > *best)
        {
            next_major = Some((release, version));
        }
    }

    UpdateCheck {
        update: same_major.map(|(release, version)| info(release, &version)),
        next_major: next_major.map(|(release, version)| info(release, &version)),
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::{pick_update, ReleaseEntry, UpdateCheck};

    fn rel(version: &str, prerelease: bool) -> ReleaseEntry {
        ReleaseEntry {
            tag_name: version.to_string(),
            html_url: format!("https://example.test/{version}"),
            draft: false,
            prerelease,
        }
    }

    fn current(version: &str) -> Version {
        Version::parse(version).expect("valid test version")
    }

    #[test]
    fn v1_user_gets_newest_v1_and_separate_v2_notice() {
        let releases = [rel("v2.0.0", false), rel("v1.1.35", false), rel("v1.1.34", false)];
        let result = pick_update(&releases, &current("1.1.33"));
        assert_eq!(result.update.unwrap().version, "1.1.35");
        assert_eq!(result.next_major.unwrap().version, "2.0.0");
    }

    #[test]
    fn v2_user_ignores_v1_patch_published_later() {
        let releases = [rel("v1.1.40", false), rel("v2.0.1", false), rel("v2.0.0", false)];
        let result = pick_update(&releases, &current("2.0.0"));
        assert_eq!(result.update.unwrap().version, "2.0.1");
        assert_eq!(result.next_major, None);
    }

    #[test]
    fn stable_user_skips_prereleases_of_any_line() {
        let releases = [rel("v2.0.0-alpha.1", true), rel("v1.1.35-rc.1", true), rel("v1.1.34", false)];
        let result = pick_update(&releases, &current("1.1.34"));
        assert_eq!(result, UpdateCheck::default());
    }

    #[test]
    fn prerelease_user_gets_next_prerelease_then_final() {
        let releases = [rel("v2.0.0-alpha.2", true), rel("v1.1.35-rc.1", true)];
        let result = pick_update(&releases, &current("2.0.0-alpha.1"));
        assert_eq!(result.update.unwrap().version, "2.0.0-alpha.2");

        let releases = [rel("v2.0.0", false), rel("v2.0.0-alpha.2", true)];
        let result = pick_update(&releases, &current("2.0.0-alpha.2"));
        assert_eq!(result.update.unwrap().version, "2.0.0");
    }

    #[test]
    fn drafts_and_missing_urls_are_ignored() {
        let mut draft = rel("v2.1.0", false);
        draft.draft = true;
        let mut missing_url = rel("v2.0.2", false);
        missing_url.html_url.clear();
        let releases = [draft, missing_url, rel("v2.0.1", false)];
        let result = pick_update(&releases, &current("2.0.0"));
        assert_eq!(result.update.unwrap().version, "2.0.1");
    }
}
