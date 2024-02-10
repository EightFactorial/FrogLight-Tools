use std::path::Path;

use cargo_metadata::Metadata as Workspace;
use froglight_data::Version;
use froglight_extractor::classmap::ClassMap;
use hashbrown::HashMap;
use proc_macro2::{Ident, Span};

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
    pub(super) version_data: HashMap<Version, ClassMap>,
}

impl DataBundle {
    pub(super) fn new(
        args: GeneratorArgs,
        config: GeneratorConfig,
        workspace: Workspace,
        version_data: HashMap<Version, ClassMap>,
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

/// Generate a module name for the given version.
pub(super) fn module_name(version: &Version) -> Ident {
    Ident::new(&format!("v{}", version.to_string().replace('.', "_")), Span::call_site())
}

/// Generate a struct name for the given version.
pub(super) fn struct_name(version: &Version) -> Ident {
    Ident::new(&format!("V{}", version.to_string().replace('.', "_")), Span::call_site())
}
