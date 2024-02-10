//! Build script for the froglight-generator crate.

use std::error::Error;

use vergen::EmitBuilder;

/// Run the build script.
pub fn main() -> Result<(), Box<dyn Error>> {
    // Generate the version information.
    EmitBuilder::builder().build_date().git_branch().git_sha(true).git_dirty(true).emit()?;

    Ok(())
}
