use crate::manifests::AssetManifest;

#[test]
fn asset_manifest_deserialize() {
    let manifest: &str = r#"{
            "objects": {
                "icons/icon_128x128.png": {
                    "hash": "b62ca8ec10d07e6bf5ac8dae0c8c1d2e6a1e3356",
                    "size": 9101
                },
                "icons/icon_16x16.png": {
                    "hash": "5ff04807c356f1beed0b86ccf659b44b9983e3fa",
                    "size": 781
                },
                "icons/icon_256x256.png": {
                    "hash": "8030dd9dc315c0381d52c4782ea36c6baf6e8135",
                    "size": 19642
                },
                "icons/icon_32x32.png": {
                    "hash": "af96f55a90eaf11b327f1b5f8834a051027dc506",
                    "size": 2063
                },
                "icons/icon_48x48.png": {
                    "hash": "b80b6e9ff01c78c624df5429e1d3dcd3d5130834",
                    "size": 3409
                }
            }
        }"#;

    let manifest: AssetManifest = serde_json::from_str(manifest).unwrap();

    assert_eq!(manifest.objects.len(), 5);

    assert_eq!(
        manifest.objects["icons/icon_128x128.png"].hash,
        "b62ca8ec10d07e6bf5ac8dae0c8c1d2e6a1e3356"
    );
    assert_eq!(manifest.objects["icons/icon_128x128.png"].size, 9101);

    assert_eq!(
        manifest.objects["icons/icon_16x16.png"].hash,
        "5ff04807c356f1beed0b86ccf659b44b9983e3fa"
    );
    assert_eq!(manifest.objects["icons/icon_16x16.png"].size, 781);
}
