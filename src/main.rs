#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[expect(clippy::let_unit_value)]
    let _data = froglight_extract::extract().await?;

    Ok(())
}
