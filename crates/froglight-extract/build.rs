//! Build script for the froglight-extract crate.
//!
//! This build script gathers information about the
//! date and git sha to mark extracted code.
use std::error::Error;

use vergen::EmitBuilder;

/// Run the build script.
pub fn main() -> Result<(), Box<dyn Error>> {
    // Gather git repository information.
    EmitBuilder::builder().build_date().git_branch().git_sha(true).git_dirty(true).emit()?;

    Ok(())
}
