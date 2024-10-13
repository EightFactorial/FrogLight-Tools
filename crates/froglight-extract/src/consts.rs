//! Constants used by the `froglight-generate` crate.

// /// The git hash of the current commit.
// ///
// /// If the repository is dirty, the hash will be suffixed with `-dirty`.
// pub(crate) const GIT_HASH: &str = {
//     if env!("VERGEN_GIT_DIRTY").as_bytes() == b"true" {
//         concat!(env!("VERGEN_GIT_SHA"), "-dirty")
//     } else {
//         env!("VERGEN_GIT_SHA")
//     }
// };

/// The git hash of the current commit.
///
/// Due to bugs in `vergen-gix`,
/// it is not possible to determine if the repository is dirty.
pub(crate) const GIT_HASH: &str = concat!(env!("VERGEN_GIT_SHA"), "-unknown");
