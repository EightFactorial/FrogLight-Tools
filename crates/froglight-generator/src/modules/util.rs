use std::{io::Write, path::Path, process::Stdio};

use cargo_metadata::Metadata as Workspace;
use froglight_data::Version;
use froglight_extractor::classmap::ClassMap;
use hashbrown::HashMap;
use proc_macro2::{Ident, Span};
use syn::File;

use crate::{command::GeneratorArgs, config::GeneratorConfig};

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

#[allow(dead_code)]
pub(super) struct DataBundle {
    pub(super) args: GeneratorArgs,
    pub(super) config: GeneratorConfig,
    pub(super) workspace: Workspace,
    pub(super) version_data: HashMap<Version, (ClassMap, serde_json::Value)>,
}

impl DataBundle {
    pub(super) fn new(
        args: GeneratorArgs,
        config: GeneratorConfig,
        workspace: Workspace,
        version_data: HashMap<Version, (ClassMap, serde_json::Value)>,
    ) -> Self {
        Self { args, config, workspace, version_data }
    }
}

/// Get the path to the package in the workspace.
pub(super) fn package_path<'a>(name: &str, workspace: &'a Workspace) -> anyhow::Result<&'a Path> {
    let package =
        workspace.workspace_packages().into_iter().find(|p| p.name == name).ok_or_else(|| {
            anyhow::anyhow!("Could not find the `{}` package in the workspace", name)
        })?;

    Ok(package.manifest_path.parent().unwrap().as_std_path())
}

/// Write a syn File to the path, formatting it with rustfmt.
///
/// Uses `prettyplease` to generate code and `rustfmt` to format it.
pub(super) fn write_formatted(file: &File, path: &Path, data: &DataBundle) -> anyhow::Result<()> {
    // Unparse the syn File
    let unparsed = prettyplease::unparse(file);

    // Create a rustfmt process
    let mut fmt = std::process::Command::new("rustfmt")
        .current_dir(data.workspace.workspace_root.as_std_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    // Write the unparsed file to rustfmt's stdin
    fmt.stdin.as_mut().unwrap().write_all(unparsed.as_bytes())?;

    // Write formatted result to the file
    let result = fmt.wait_with_output()?;
    let mut file = std::fs::File::create(path)?;
    file.write_all(result.stdout.as_slice())?;

    Ok(())
}

/// Generate a module name for the given version.
pub(super) fn module_name(version: &Version) -> Ident {
    Ident::new(&format!("v{}", version.to_string().replace('.', "_")), Span::call_site())
}

/// Generate a struct name for the given version.
pub(super) fn struct_name(version: &Version) -> Ident {
    Ident::new(&format!("V{}", version.to_string().replace('.', "_")), Span::call_site())
}
