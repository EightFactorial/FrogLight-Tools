//! TODO

use froglight_tool_macros::Dependency;
use serde::{Deserialize, Serialize};

use crate::{container::DependencyContainer, version::Version};

/// The yarn maven repository.
///
/// Contains information on all yarn builds.
#[derive(Debug, Clone, PartialEq, Eq, Dependency, Serialize, Deserialize)]
#[dep(path = crate, retrieve = Self::retrieve)]
pub struct YarnMaven {
    /// The versioning information.
    pub versioning: YarnVersioning,
}

impl YarnMaven {
    const URL_TEMPLATE: &str =
        "https://maven.fabricmc.net/net/fabricmc/yarn/{BUILD}/yarn-{BUILD}-mergedv2.jar";

    /// Returns the URL for latest build for the given [`Version`].
    #[must_use]
    pub fn get_url(&self, version: &Version) -> Option<String> {
        self.get_build(version).map(|build| Self::URL_TEMPLATE.replace("{BUILD}", &build))
    }

    /// Returns the latest build for the given [`Version`].
    #[must_use]
    pub fn get_build(&self, version: &Version) -> Option<String> {
        let version_str = version.to_short_string();

        let builds = self.iter().filter(|build| build.starts_with(&version_str));
        let builds = builds.filter_map(|str| match str.split_once("+build.") {
            Some((ver, build)) if ver == version_str => Some(build),
            _ => None,
        });

        let mut builds: Vec<u32> = builds.filter_map(|b| b.parse().ok()).collect();
        builds.sort_unstable();

        builds.last().map(|n| format!("{version_str}+build.{n}"))
    }
}

/// The versioning information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YarnVersioning {
    /// The list of versions.
    pub versions: YarnVersions,
}

/// The list of versions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YarnVersions {
    /// The list of versions.
    pub version: Vec<String>,
}

impl YarnMaven {
    const FILENAME: &str = "yarn-maven-metadata.xml";
    const URL: &str = "https://maven.fabricmc.net/net/fabricmc/yarn/maven-metadata.xml";

    async fn retrieve(deps: &mut DependencyContainer) -> anyhow::Result<Self> {
        let path = deps.cache.join(Self::FILENAME);
        if tokio::fs::try_exists(&path).await? {
            tracing::debug!("Reading \"{}\"", path.display());

            // Read the file from disk
            let content = tokio::fs::read_to_string(&path).await?;
            quick_xml::de::from_str(&content).map_err(Into::into)
        } else {
            tracing::debug!("Retrieving \"{}\"", Self::URL);

            // Download the file and save it to disk
            let response = deps.client.get(Self::URL).send().await?.bytes().await?;
            tokio::fs::write(&path, &response).await?;
            quick_xml::de::from_reader(&mut std::io::Cursor::new(response)).map_err(Into::into)
        }
    }
}

#[test]
#[cfg(test)]
fn parse() {
    let maven: YarnMaven = quick_xml::de::from_str(TRIMMED_EXAMPLE).unwrap();
    assert_eq!(maven.len(), 37);
    assert_eq!(maven[0], "1.21.4+build.1");
    assert_eq!(maven[36], "25w05a+build.4");
}

#[cfg(test)]
const TRIMMED_EXAMPLE: &str = r"<metadata>
<groupId>net.fabricmc</groupId>
<artifactId>yarn</artifactId>
<versioning>
<latest>25w05a+build.4</latest>
<release>25w05a+build.4</release>
<versions>
<version>1.21.4+build.1</version>
<version>1.21.4+build.2</version>
<version>1.21.4+build.3</version>
<version>1.21.4+build.4</version>
<version>1.21.4+build.5</version>
<version>1.21.4+build.6</version>
<version>1.21.4+build.7</version>
<version>1.21.4+build.8</version>
<version>25w02a+build.1</version>
<version>25w02a+build.2</version>
<version>25w02a+build.3</version>
<version>25w02a+build.4</version>
<version>25w02a+build.5</version>
<version>25w02a+build.6</version>
<version>25w02a+build.7</version>
<version>25w02a+build.8</version>
<version>25w02a+build.9</version>
<version>25w02a+build.10</version>
<version>25w02a+build.11</version>
<version>25w02a+build.12</version>
<version>25w03a+build.1</version>
<version>25w03a+build.2</version>
<version>25w03a+build.3</version>
<version>25w04a+build.1</version>
<version>25w04a+build.2</version>
<version>25w04a+build.3</version>
<version>25w04a+build.4</version>
<version>25w04a+build.5</version>
<version>25w04a+build.6</version>
<version>25w04a+build.7</version>
<version>25w04a+build.8</version>
<version>25w04a+build.9</version>
<version>25w04a+build.10</version>
<version>25w05a+build.1</version>
<version>25w05a+build.2</version>
<version>25w05a+build.3</version>
<version>25w05a+build.4</version>
</versions>
<lastUpdated>20250130213348</lastUpdated>
</versioning>
</metadata>";

impl std::ops::Deref for YarnMaven {
    type Target = YarnVersioning;
    fn deref(&self) -> &Self::Target { &self.versioning }
}
impl std::ops::DerefMut for YarnMaven {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.versioning }
}

impl std::ops::Deref for YarnVersioning {
    type Target = YarnVersions;
    fn deref(&self) -> &Self::Target { &self.versions }
}
impl std::ops::DerefMut for YarnVersioning {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.versions }
}

impl std::ops::Deref for YarnVersions {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target { &self.version }
}
impl std::ops::DerefMut for YarnVersions {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.version }
}
