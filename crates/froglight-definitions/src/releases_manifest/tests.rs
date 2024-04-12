use crate::{MinecraftVersion, ReleasesManifest};

#[test]
fn releases_deserialize() {
    let manifest: &str = r#"{
        "latest": {
            "release": "1.20.4",
            "snapshot": "24w03b"
        },
        "versions": [
            {
                "id": "24w03b",
                "type": "snapshot",
                "url": "https://piston-meta.mojang.com/v1/packages/ea3ab7762af9fd43565a5d8d96652899a4dc6303/24w03b.json",
                "time": "2024-01-18T12:49:51+00:00",
                "releaseTime": "2024-01-18T12:42:37+00:00",
                "sha1":	"ea3ab7762af9fd43565a5d8d96652899a4dc6303"
            },
            {
                "id": "1.20.4",
                "type": "release",
                "url": "https://piston-meta.mojang.com/v1/packages/c98adde5094a3041f486b4d42d0386cf87310559/1.20.4.json",
                "time":	"2024-01-18T12:24:32+00:00",
                "releaseTime": "2023-12-07T12:56:20+00:00",
                "sha1":	"ea3ab7762af9fd43565a5d8d96652899a4dc6303"
            }
        ]
    }"#;
    let manifest: ReleasesManifest = serde_json::from_str(manifest).unwrap();

    let release = MinecraftVersion::new_release(1, 20, 4);
    let snapshot = MinecraftVersion::new_snapshot(24, 3, 'b').unwrap();

    // Test ReleasesLatest
    {
        assert!(manifest.latest.release.is_same(&release));
        assert!(manifest.latest.snapshot.is_same(&snapshot));
    }

    // Test ReleasesManifestEntry array
    {
        assert_eq!(manifest.versions.len(), 2);

        assert!(manifest.versions[0].id.is_same(&snapshot));
        assert_eq!(manifest.versions[0].kind, "snapshot");

        assert!(manifest.versions[1].id.is_same(&release));
        assert_eq!(manifest.versions[1].kind, "release");
    }
}
