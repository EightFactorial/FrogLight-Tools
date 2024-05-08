use crate::manifests::ReleaseManifest;

#[test]
fn release_manifest_deserialize() {
    let manifest: &str = r#"{
        "assetIndex": {
          "id": "16",
          "sha1": "2424100100917826a5933159802456d10f50d99a",
          "size": 445177,
          "totalSize": 630830282,
          "url": "https://piston-meta.mojang.com/v1/packages/2424100100917826a5933159802456d10f50d99a/16.json"
        },
        "downloads": {
          "client": {
            "sha1": "05b6f1c6b46a29d6ea82b4e0d42190e42402030f",
            "size": 26565641,
            "url": "https://piston-data.mojang.com/v1/objects/05b6f1c6b46a29d6ea82b4e0d42190e42402030f/client.jar"
          },
          "client_mappings": {
            "sha1": "de46c8f33d7826eb83e8ef0e9f80dc1f08cb9498",
            "size": 9422442,
            "url": "https://piston-data.mojang.com/v1/objects/de46c8f33d7826eb83e8ef0e9f80dc1f08cb9498/client.txt"
          },
          "server": {
            "sha1": "145ff0858209bcfc164859ba735d4199aafa1eea",
            "size": 51420480,
            "url": "https://piston-data.mojang.com/v1/objects/145ff0858209bcfc164859ba735d4199aafa1eea/server.jar"
          },
          "server_mappings": {
            "sha1": "9e96100f573a46ef44caab3e716d5eb974594bb7",
            "size": 7283803,
            "url": "https://piston-data.mojang.com/v1/objects/9e96100f573a46ef44caab3e716d5eb974594bb7/server.txt"
          }
        }
      }"#;

    let manifest: ReleaseManifest = serde_json::from_str(manifest).unwrap();

    assert_eq!(manifest.asset_index.sha1, "2424100100917826a5933159802456d10f50d99a");
    assert_eq!(manifest.downloads.client.sha1, "05b6f1c6b46a29d6ea82b4e0d42190e42402030f");
    assert_eq!(manifest.downloads.client_mappings.sha1, "de46c8f33d7826eb83e8ef0e9f80dc1f08cb9498");
    assert_eq!(manifest.downloads.server.sha1, "145ff0858209bcfc164859ba735d4199aafa1eea");
    assert_eq!(manifest.downloads.server_mappings.sha1, "9e96100f573a46ef44caab3e716d5eb974594bb7");
}
