//! TODO

use froglight_tool_macros::Dependency;
use serde::{Deserialize, Serialize};

use crate::{container::DependencyContainer, version::Version};

/// The fabric maven repository.
///
/// Contains information on all fabric builds.
#[derive(Debug, Clone, PartialEq, Eq, Dependency, Serialize, Deserialize)]
#[dep(path = crate, retrieve = Self::retrieve)]
pub struct FabricMaven {
    /// The versioning information.
    pub versioning: FabricVersioning,
}

impl FabricMaven {
    /// Returns the latest build for the given [`Version`].
    #[must_use]
    pub fn get_build(&self, version: &Version) -> Option<String> {
        let version_str = version.to_short_string();

        let builds = self.iter().filter(|build| build.ends_with(&version_str));
        let builds = builds.filter_map(|str| match str.split_once('+') {
            Some((build, ver)) if ver == version_str => Some(build),
            _ => None,
        });

        let mut builds: Vec<semver::Version> =
            builds.filter_map(|b| Some(semver::Version::parse(b).unwrap())).collect();
        builds.sort_unstable();

        builds.last().map(|n| format!("{n}+{version_str}"))
    }
}

/// The versioning information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FabricVersioning {
    /// The list of versions.
    pub versions: FabricVersions,
}

/// The list of versions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FabricVersions {
    /// The list of versions.
    pub version: Vec<String>,
}

impl FabricMaven {
    const FILENAME: &str = "fabric-maven-metadata.xml";
    const URL: &str =
        "https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/maven-metadata.xml";

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
    let maven: FabricMaven = quick_xml::de::from_str(TRIMMED_EXAMPLE).unwrap();

    assert_eq!(maven.len(), 105);
    assert_eq!(maven[0], "0.3.0-pre+build.135");
    assert_eq!(maven[104], "0.123.2+1.21.6");

    let version = Version::new_release(1, 21, 4);
    assert_eq!(maven.get_build(&version).unwrap(), "0.119.2+1.21.4");

    let version = Version::new_release(1, 21, 6);
    assert_eq!(maven.get_build(&version).unwrap(), "0.123.2+1.21.6");
}

#[cfg(test)]
const TRIMMED_EXAMPLE: &str = r"<metadata>
<groupId>net.fabricmc.fabric-api</groupId>
<artifactId>fabric-api</artifactId>
<versioning>
<latest>0.123.2+1.21.6</latest>
<release>0.123.2+1.21.6</release>
<versions>
<version>0.3.0-pre+build.135</version>
<version>0.3.0-pre+build.137</version>
<version>0.3.0-pre+build.138</version>
<version>0.3.0-pre+build.139</version>
<version>0.3.0-pre+build.140</version>
<version>0.3.0-pre+build.141</version>
<version>0.3.0-pre+build.142</version>
<version>0.3.0-pre+build.143</version>
<version>0.3.0-pre+build.144</version>
<version>0.3.0-pre+build.147</version>
<version>0.3.0-pre+build.148</version>
<version>0.3.0-pre+build.149</version>
<version>0.3.0-pre+build.150</version>
<version>0.3.0-pre+build.151</version>
<version>0.3.0-pre+build.153</version>
<version>0.3.0-pre+build.155</version>
<version>0.3.0-pre+build.156</version>
<version>0.3.0-pre+build.157</version>
<version>0.3.0-pre+build.158</version>
<version>0.3.0-pre+build.161</version>
<version>0.3.0-pre+build.162</version>
<version>0.3.0-pre+build.163</version>
<version>0.3.0-pre+build.164</version>
<version>0.3.0-pre+build.165</version>
<version>0.3.0-pre+build.166</version>
<version>0.3.0-pre+build.167</version>
<version>0.3.0-pre+build.168</version>
<version>0.3.0-pre+build.169</version>
<version>0.3.0+build.170</version>
<version>0.3.0+build.171</version>
<version>0.3.0+build.172</version>
<version>0.3.0+build.173</version>
<version>0.3.0+build.174</version>
<version>0.3.0+build.175</version>
<version>0.3.0+build.176</version>
<version>0.3.0+build.177</version>
<version>0.3.0+build.178</version>
<version>0.3.0+build.179</version>
<version>0.3.0+build.180</version>
<version>0.3.0+build.181</version>
<version>0.3.0+build.183</version>
<version>0.3.0+build.184</version>
<version>0.3.0+build.185</version>
<version>0.3.0+build.186</version>
<version>0.3.0+build.187</version>
<version>0.3.0+build.188</version>
<version>0.3.0+build.191</version>
<version>0.3.0+build.192</version>
<version>0.3.0+build.194</version>
<version>0.3.0+build.196</version>
<version>0.3.0+build.197</version>
<version>0.3.0+build.198</version>
<version>0.3.0+build.200</version>
<version>0.3.0+build.206</version>
<version>0.3.0+build.207</version>
<version>0.3.1+build.208</version>
<version>0.3.2+build.212-1.15</version>
<version>0.115.1+1.21.1</version>
<version>0.118.0+1.21.4</version>
<version>0.118.0+1.21.5</version>
<version>0.118.1+1.21.5</version>
<version>0.118.2+1.21.5</version>
<version>0.118.3+1.21.5</version>
<version>0.118.4+1.21.5</version>
<version>0.118.5+1.21.4</version>
<version>0.118.5+1.21.5</version>
<version>0.118.6+1.21.5</version>
<version>0.92.4+1.20.1</version>
<version>0.115.2+1.21.1</version>
<version>0.119.0+1.21.4</version>
<version>0.119.0+1.21.5</version>
<version>0.119.1+1.21.5</version>
<version>0.92.5+1.20.1</version>
<version>0.115.3+1.21.1</version>
<version>0.119.2+1.21.4</version>
<version>0.119.2+1.21.5</version>
<version>0.119.3+1.21.5</version>
<version>0.119.4+1.21.5</version>
<version>0.119.5+1.21.5</version>
<version>0.115.4+1.21.1</version>
<version>0.119.6+1.21.5</version>
<version>0.119.7+25w14craftmine</version>
<version>0.119.8+25w14craftmine</version>
<version>0.119.9+1.21.5</version>
<version>0.119.9+25w14craftmine</version>
<version>0.119.10+1.21.6</version>
<version>0.119.10+25w14craftmine</version>
<version>0.120.0+1.21.5</version>
<version>0.120.0+1.21.6</version>
<version>0.120.1+1.21.6</version>
<version>0.120.2+1.21.6</version>
<version>0.115.5+1.21.1</version>
<version>0.121.0+1.21.5</version>
<version>0.121.0+1.21.6</version>
<version>0.115.6+1.21.1</version>
<version>0.121.1+1.21.6</version>
<version>0.121.2+1.21.6</version>
<version>0.122.0+1.21.5</version>
<version>0.122.0+1.21.6</version>
<version>0.116.0+1.21.1</version>
<version>0.123.0+1.21.5</version>
<version>0.123.0+1.21.6</version>
<version>0.123.1+1.21.6</version>
<version>0.123.2+1.21.5</version>
<version>0.123.2+1.21.6</version>
</versions>
<lastUpdated>20250508203831</lastUpdated>
</versioning>
</metadata>";

impl std::ops::Deref for FabricMaven {
    type Target = FabricVersioning;
    fn deref(&self) -> &Self::Target { &self.versioning }
}
impl std::ops::DerefMut for FabricMaven {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.versioning }
}

impl std::ops::Deref for FabricVersioning {
    type Target = FabricVersions;
    fn deref(&self) -> &Self::Target { &self.versions }
}
impl std::ops::DerefMut for FabricVersioning {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.versions }
}

impl std::ops::Deref for FabricVersions {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target { &self.version }
}
impl std::ops::DerefMut for FabricVersions {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.version }
}
