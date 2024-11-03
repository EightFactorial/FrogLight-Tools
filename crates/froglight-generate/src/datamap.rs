use froglight_parse::{
    file::{DataPath, FileTrait, GeneratorData, VersionInfo, VersionManifest, VersionProtocol},
    Version,
};
use hashbrown::HashMap;
use reqwest::Client;

use crate::{cli::CliArgs, config::Config};

#[derive(Debug, PartialEq, Eq)]
pub struct DataMap {
    pub manifest: VersionManifest,
    pub datapath: DataPath,

    pub version_data: HashMap<Version, DataSet>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSet {
    pub info: VersionInfo,
    pub generated: GeneratorData,

    pub proto: VersionProtocol,
}

impl DataMap {
    pub async fn new(args: &CliArgs, config: &Config) -> anyhow::Result<Self> {
        let cache = args.cache.as_ref().unwrap();
        let client = Client::new();

        // Get the VersionManifest and DataPath
        let Some(any) = config.iter().next() else {
            anyhow::bail!("No versions specified in the configuration file.");
        };
        let man = VersionManifest::fetch(&any.target, cache, &(), args.redownload, &client).await?;
        let dat = DataPath::fetch(&any.target, cache, &(), args.redownload, &client).await?;

        // Fetch data for all versions
        let mut version_data = HashMap::new();
        for version in config.iter() {
            // Get the VersionInfo
            let info =
                VersionInfo::fetch(&version.target, cache, &man, args.redownload, &client).await?;
            // Get the GeneratorData
            let generated =
                GeneratorData::fetch(&version.target, cache, &info, args.redownload, &client)
                    .await?;

            // Get the VersionProtocol
            let proto =
                VersionProtocol::fetch(&version.target, cache, &dat, args.redownload, &client)
                    .await?;

            version_data.insert(version.base.clone(), DataSet { info, generated, proto });
        }

        Ok(Self { manifest: man, datapath: dat, version_data })
    }
}
