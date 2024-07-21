//! Build script for the froglight-extract crate.
//!
//! This build script gathers information about the
//! date and git sha to mark extracted code.
use std::error::Error;

use vergen_gix::{BuildBuilder, Emitter, GixBuilder};

/// Gather git repository and build information.
pub fn main() -> Result<(), Box<dyn Error>> {
    let build = BuildBuilder::default().build_date(true).build()?;
    let gix = GixBuilder::default().branch(true).dirty(true).sha(true).build()?;

    Emitter::new().add_instructions(&build)?.add_instructions(&gix)?.emit()?;

    Ok(())
}
