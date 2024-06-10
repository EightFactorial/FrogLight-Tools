use std::sync::LazyLock;

use regex::Regex;

use crate::MinecraftVersion;

// -----------------------------------------------------------------------------

/// The [`Regex`] for [`MinecraftVersion::Release`].
static RELEASE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(RELEASE_REGEX_STR).unwrap());
/// The string for [`RELEASE_REGEX`].
static RELEASE_REGEX_STR: &str = r"^(\d+)\.(\d+)(\.(\d+))?$";
/// The capture groups for [`RELEASE_REGEX`].
static RELEASE_REGEX_GROUPS: [usize; 3] = [1, 2, 4];

pub(crate) fn parse_release(ver: &str) -> Option<MinecraftVersion> {
    let caps = RELEASE_REGEX.captures(ver)?;
    let major = caps.get(RELEASE_REGEX_GROUPS[0])?.as_str().parse().ok()?;
    let minor = caps.get(RELEASE_REGEX_GROUPS[1])?.as_str().parse().ok()?;

    let patch = if let Some(cap) = caps.get(RELEASE_REGEX_GROUPS[2]) {
        cap.as_str().parse().ok()?
    } else {
        0
    };

    Some(MinecraftVersion::new_release(major, minor, patch))
}

// -----------------------------------------------------------------------------

/// The [`Regex`] for [`MinecraftVersion::ReleaseCandidate`].
static RELEASE_CANDIDATE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(RELEASE_CANDIDATE_REGEX_STR).unwrap());
/// The string for [`RELEASE_CANDIDATE_REGEX`].
static RELEASE_CANDIDATE_REGEX_STR: &str = r"^(\d+)\.(\d+)(\.(\d+))?-rc(\d+)$";
/// The capture groups for [`RELEASE_CANDIDATE_REGEX`].
static RELEASE_CANDIDATE_REGEX_GROUPS: [usize; 4] = [1, 2, 4, 5];

pub(crate) fn parse_release_candidate(ver: &str) -> Option<MinecraftVersion> {
    let caps = RELEASE_CANDIDATE_REGEX.captures(ver)?;
    let major = caps.get(RELEASE_CANDIDATE_REGEX_GROUPS[0])?.as_str().parse().ok()?;
    let minor = caps.get(RELEASE_CANDIDATE_REGEX_GROUPS[1])?.as_str().parse().ok()?;

    let patch = if let Some(cap) = caps.get(RELEASE_CANDIDATE_REGEX_GROUPS[2]) {
        cap.as_str().parse().ok()?
    } else {
        0
    };

    let rc = caps.get(RELEASE_CANDIDATE_REGEX_GROUPS[3])?.as_str().parse().ok()?;
    Some(MinecraftVersion::new_release_candidate(major, minor, patch, rc).unwrap())
}

// -----------------------------------------------------------------------------

/// The [`Regex`] for [`MinecraftVersion::PreRelease`].
static PRE_RELEASE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(PRE_RELEASE_REGEX_STR).unwrap());
/// The string for [`PRE_RELEASE_REGEX`].
static PRE_RELEASE_REGEX_STR: &str = r"^(\d+)\.(\d+)(\.(\d+))?-pre(\d+)$";
/// The capture groups for [`PRE_RELEASE_REGEX`].
static PRE_RELEASE_REGEX_GROUPS: [usize; 4] = [1, 2, 4, 5];

pub(crate) fn parse_pre_release(ver: &str) -> Option<MinecraftVersion> {
    let caps = PRE_RELEASE_REGEX.captures(ver)?;
    let major = caps.get(PRE_RELEASE_REGEX_GROUPS[0])?.as_str().parse().ok()?;
    let minor = caps.get(PRE_RELEASE_REGEX_GROUPS[1])?.as_str().parse().ok()?;

    let patch = if let Some(cap) = caps.get(PRE_RELEASE_REGEX_GROUPS[2]) {
        cap.as_str().parse().ok()?
    } else {
        0
    };

    let pre = caps.get(PRE_RELEASE_REGEX_GROUPS[3])?.as_str().parse().ok()?;
    Some(MinecraftVersion::new_pre_release(major, minor, patch, pre).unwrap())
}

// -----------------------------------------------------------------------------

/// The [`Regex`] for [`MinecraftVersion::Snapshot`].
static SNAPSHOT_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(SNAPSHOT_REGEX_STR).unwrap());
/// The string for [`SNAPSHOT_REGEX`].
static SNAPSHOT_REGEX_STR: &str = r"^(\d\d)w(\d\d)([a-z])$";
/// The capture groups for [`SNAPSHOT_REGEX`].
static SNAPSHOT_REGEX_GROUPS: [usize; 3] = [1, 2, 3];

pub(crate) fn parse_snapshot(ver: &str) -> Option<MinecraftVersion> {
    let caps = SNAPSHOT_REGEX.captures(ver)?;
    let year = caps.get(SNAPSHOT_REGEX_GROUPS[0])?.as_str().parse().ok()?;
    let week = caps.get(SNAPSHOT_REGEX_GROUPS[1])?.as_str().parse().ok()?;
    let patch = caps.get(SNAPSHOT_REGEX_GROUPS[2])?.as_str().parse().ok()?;
    Some(MinecraftVersion::new_snapshot(year, week, patch).unwrap())
}
