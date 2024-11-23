use std::path::Path;

use froglight_parse::{
    file::{
        DataPath, FileTrait, GeneratorData, VersionBlocks, VersionInfo, VersionManifest,
        VersionProtocol,
    },
    Version,
};
use hashbrown::HashMap;
use reqwest::Client;

use crate::{
    cli::CliArgs,
    config::{Config, VersionTuple},
};

/// A map of data containing the [`VersionManifest`], [`DataPath`],
/// and data for each [`Version`] when created.
#[derive(Debug, PartialEq)]
pub struct DataMap {
    /// The version manifest.
    pub manifest: VersionManifest,
    /// The data path.
    pub datapath: DataPath,

    /// The data for each version.
    pub version_data: HashMap<Version, DataSet>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataSet {
    pub info: VersionInfo,
    pub generated: GeneratorData,

    pub blocks: VersionBlocks,
    pub proto: VersionProtocol,
}

impl DataMap {
    /// Create a new [`DataMap`] from the given [`CliArgs`] and [`Config`].
    #[expect(clippy::missing_errors_doc, clippy::missing_panics_doc)]
    pub async fn new(args: &CliArgs, config: &Config) -> anyhow::Result<Self> {
        let cache = args.cache.as_ref().unwrap();
        let redownload = args.redownload;

        Self::new_from(cache, &config.version, redownload).await
    }

    /// Create a new [`DataMap`] from the given
    /// cache path, versions, and redownload flag.
    #[expect(clippy::missing_errors_doc)]
    pub async fn new_from(
        cache: &Path,
        versions: &[VersionTuple],
        redownload: bool,
    ) -> anyhow::Result<Self> {
        let client = Client::new();

        // Get the VersionManifest and DataPath
        let Some(VersionTuple { base: any, .. }) = versions.iter().next() else {
            anyhow::bail!("No versions specified in the configuration file.");
        };
        let man = VersionManifest::fetch(any, cache, &(), redownload, &client).await?;
        let dat = DataPath::fetch(any, cache, &(), redownload, &client).await?;

        // Fetch data for all versions
        let mut version_data = HashMap::new();
        for VersionTuple { base, target } in versions {
            // Get the VersionInfo
            tracing::trace!("VersionInfo: {base}");
            let info = VersionInfo::fetch(target, cache, &man, redownload, &client).await?;
            // Get the GeneratorData
            tracing::trace!("GeneratorData: {base}");
            let generated = GeneratorData::fetch(target, cache, &info, redownload, &client).await?;

            // Get the VersionBlocks
            tracing::trace!("VersionBlocks: {base}");
            let blocks = VersionBlocks::fetch(target, cache, &dat, redownload, &client).await?;
            // Get the VersionProtocol
            tracing::trace!("VersionProtocol: {base}");
            let proto = VersionProtocol::fetch(target, cache, &dat, redownload, &client).await?;

            // Create and insert the DataSet
            version_data.insert(base.clone(), DataSet { info, generated, blocks, proto });
        }

        Ok(Self { manifest: man, datapath: dat, version_data })
    }
}
