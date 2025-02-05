#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use std::path::PathBuf;

use froglight_dependency::{
    container::{DependencyContainer, SharedDependencies},
    dependency::minecraft::{DataGenerator, DecompiledJar},
    version::Version,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    froglight_extract::logging();

    let version = Version::new_release(1, 21, 4);

    let path = std::env::var("CARGO_MANIFEST_DIR").map(PathBuf::from)?;
    let dependencies = SharedDependencies::new(path.join("target/froglight-tools"));

    #[expect(clippy::let_unit_value)]
    let _data = froglight_extract::extract(version.clone(), None, dependencies.clone()).await?;

    {
        let mut deps = dependencies.write().await;

        deps.get_or_retrieve::<DecompiledJar>().await?;
        deps.scoped_fut::<DecompiledJar, anyhow::Result<()>>(
            async |jar: &mut DecompiledJar, deps: &mut DependencyContainer| {
                jar.get_client(&version, deps).await?;
                Ok(())
            },
        )
        .await?;

        deps.get_or_retrieve::<DataGenerator>().await?;
        deps.scoped_fut::<DataGenerator, anyhow::Result<()>>(
            async |gen: &mut DataGenerator, deps: &mut DependencyContainer| {
                gen.get_version(&version, deps).await?;
                Ok(())
            },
        )
        .await?;
    }

    Ok(())
}
