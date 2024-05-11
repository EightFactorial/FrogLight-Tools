use proptest::prelude::*;

use super::{regex, MinecraftVersion};

proptest! {
    #[test]
    fn parse_release(major in 0u64.., minor in 0u64.., patch in proptest::option::of(0u64..)) {
        let version = if let Some(patch) = patch { format!("{major}.{minor}.{patch}") } else { format!("{major}.{minor}") };
        let Some(MinecraftVersion::Release(release)) = regex::parse_release(&version) else {
            panic!("Failed to parse MinecraftVersion::Release -> `{version}`")
        };

        assert_eq!(release.major, major);
        assert_eq!(release.minor, minor);
        assert_eq!(release.patch, patch.unwrap_or_default());
    }

    #[test]
    fn parse_release_candidate(major in 0u64.., minor in 0u64.., patch in proptest::option::of(0u64..), rc in 0u64..) {
        let version = if let Some(patch) = patch { format!("{major}.{minor}.{patch}-rc{rc}") } else { format!("{major}.{minor}-rc{rc}") };
        let Some(MinecraftVersion::ReleaseCandidate(release_candidate)) = regex::parse_release_candidate(&version) else {
            panic!("Failed to parse MinecraftVersion::ReleaseCandidate -> `{version}`")
        };

        assert_eq!(release_candidate.major, major);
        assert_eq!(release_candidate.minor, minor);
        assert_eq!(release_candidate.patch, patch.unwrap_or_default());
        assert_eq!(release_candidate.pre.as_str(), rc.to_string());
    }

    #[test]
    fn parse_pre_release(major in 0u64.., minor in 0u64.., patch in proptest::option::of(0u64..), pre in 0u64..) {
        let version = if let Some(patch) = patch { format!("{major}.{minor}.{patch}-pre{pre}") } else { format!("{major}.{minor}-pre{pre}") };
        let Some(MinecraftVersion::PreRelease(pre_release)) = regex::parse_pre_release(&version) else {
            panic!("Failed to parse MinecraftVersion::PreRelease -> `{version}`")
        };

        assert_eq!(pre_release.major, major);
        assert_eq!(pre_release.minor, minor);
        assert_eq!(pre_release.patch, patch.unwrap_or_default());
        assert_eq!(pre_release.pre.as_str(), pre.to_string());
    }

    #[test]
    fn parse_snapshot(year in 0u64..99u64, week in 0u64..99u64, patch in "[a-z]") {
        let version = format!("{year:02}w{week:02}{patch}");
        let Some(MinecraftVersion::Snapshot(snapshot)) = regex::parse_snapshot(&version) else {
            panic!("Failed to parse MinecraftVersion::Snapshot -> `{version}`")
        };

        assert_eq!(snapshot.major, year);
        assert_eq!(snapshot.minor, week);
        assert_eq!(snapshot.pre.as_str(), patch);
    }
}

/// A list of example releases.
const EXAMPLE_RELEASES: [&str; 13] = [
    "1.18", "1.18.1", "1.18.2", "1.19", "1.19.1", "1.19.2", "1.19.3", "1.19.4", "1.20", "1.20.1",
    "1.20.2", "1.20.3", "1.20.4",
];

#[test]
fn release_ordering() {
    let releases: Vec<MinecraftVersion> =
        EXAMPLE_RELEASES.iter().map(|&version| regex::parse_release(version).unwrap()).collect();

    for (index, release) in releases.iter().enumerate() {
        for (other_index, other_release) in releases.iter().enumerate() {
            assert_eq!(
                release.try_compare(other_release).unwrap(),
                index.cmp(&other_index),
                "Error comparing `{release:?}` ({index}) with `{other_release:?}` ({other_index})"
            );
        }
    }
}

/// A list of example release candidates.
const EXAMPLE_RELEASE_CANDIDATES: [&str; 17] = [
    "1.18-rc1",
    "1.18-rc2",
    "1.18-rc3",
    "1.18-rc4",
    "1.18.1-rc1",
    "1.18.1-rc2",
    "1.18.1-rc3",
    "1.18.2-rc1",
    "1.19-rc1",
    "1.19-rc2",
    "1.19.1-rc1",
    "1.19.1-rc2",
    "1.19.2-rc1",
    "1.19.2-rc2",
    "1.19.3-rc1",
    "1.19.3-rc2",
    "1.19.3-rc3",
];

#[test]
fn release_candidate_ordering() {
    let release_candidates: Vec<MinecraftVersion> = EXAMPLE_RELEASE_CANDIDATES
        .iter()
        .map(|&version| regex::parse_release_candidate(version).unwrap())
        .collect();

    for (index, release_candidate) in release_candidates.iter().enumerate() {
        for (other_index, other_release_candidate) in release_candidates.iter().enumerate() {
            assert_eq!(
                release_candidate.try_compare(other_release_candidate).unwrap(),
                index.cmp(&other_index),
                "Error comparing `{release_candidate:?}` ({index}) with `{other_release_candidate:?}` ({other_index})"
            );
        }
    }
}

/// A list of example pre-releases.
const EXAMPLE_PRE_RELEASES: [&str; 23] = [
    "1.18-pre1",
    "1.18-pre2",
    "1.18-pre3",
    "1.18-pre4",
    "1.18-pre5",
    "1.18-pre6",
    "1.18-pre7",
    "1.18-pre8",
    "1.18.1-pre1",
    "1.18.2-pre1",
    "1.18.2-pre2",
    "1.18.2-pre3",
    "1.19-pre1",
    "1.19-pre2",
    "1.19-pre3",
    "1.19-pre4",
    "1.19-pre5",
    "1.19.1-pre1",
    "1.19.1-pre2",
    "1.19.1-pre3",
    "1.19.1-pre4",
    "1.19.1-pre5",
    "1.19.1-pre6",
];

#[test]
fn pre_release_ordering() {
    let pre_releases: Vec<MinecraftVersion> = EXAMPLE_PRE_RELEASES
        .iter()
        .map(|&version| regex::parse_pre_release(version).unwrap())
        .collect();

    for (index, pre_release) in pre_releases.iter().enumerate() {
        for (other_index, other_pre_release) in pre_releases.iter().enumerate() {
            assert_eq!(
                pre_release.try_compare(other_pre_release).unwrap(),
                index.cmp(&other_index),
                "Error comparing `{pre_release:?}` ({index}) with `{other_pre_release:?}` ({other_index})"
            );
        }
    }
}

/// A list of example snapshots.
const EXAMPLE_SNAPSHOTS: [&str; 18] = [
    "22w11a", "22w12a", "22w13a", "22w14a", "22w15a", "22w16a", "22w16b", "22w17a", "22w18a",
    "22w19a", "22w24a", "22w42a", "22w43a", "22w44a", "22w45a", "22w46a", "23w03a", "23w04a",
];

#[test]
fn snapshot_ordering() {
    let snapshots: Vec<MinecraftVersion> =
        EXAMPLE_SNAPSHOTS.iter().map(|&version| regex::parse_snapshot(version).unwrap()).collect();

    for (index, snapshot) in snapshots.iter().enumerate() {
        for (other_index, other_snapshot) in snapshots.iter().enumerate() {
            assert_eq!(
                snapshot.try_compare(other_snapshot).unwrap(),
                index.cmp(&other_index),
                "Error comparing `{snapshot:?}` ({index}) with `{other_snapshot:?}` ({other_index})",
            );
        }
    }
}
