#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use froglight_dependency::{container::SharedDependencies, version::Version};

#[tokio::main]
#[expect(clippy::let_unit_value)]
async fn main() -> anyhow::Result<()> {
    froglight_extract::logging();

    let version = Version::new_release(1, 21, 4);
    let _data = froglight_extract::extract(version, None, SharedDependencies::default()).await?;

    Ok(())
}
