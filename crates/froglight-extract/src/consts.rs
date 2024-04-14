//! Constants used by the `froglight-generate` crate.
#![allow(dead_code)]

/// The git hash of the current commit.
///
/// If the repository is dirty, the hash will be suffixed with `-dirty`.
pub(crate) const GIT_HASH: &str = {
    if env!("VERGEN_GIT_DIRTY").as_bytes()[0] == b't' {
        concat!(env!("VERGEN_GIT_SHA"), "-dirty")
    } else {
        env!("VERGEN_GIT_SHA")
    }
};
