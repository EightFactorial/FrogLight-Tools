use crate::{manifests::YarnManifest, MinecraftVersion};

#[test]
fn version_manifest_deserialize() {
    let manifest: &str = r"
    <metadata>
        <versioning>
            <latest>1.21-pre4+build.2</latest>
            <release>1.21-pre4+build.2</release>
            <versions>
                <version>24w14potato+build.1</version>
                <version>24w14potato+build.2</version>
                <version>24w14potato+build.3</version>
                <version>24w14potato+build.4</version>
                <version>24w14a+build.1</version>
                <version>24w14a+build.2</version>
                <version>24w14a+build.3</version>
                <version>24w14a+build.4</version>
                <version>24w14a+build.5</version>
                <version>24w14a+build.6</version>
                <version>1.20.5-pre1+build.2</version>
                <version>1.20.5-pre1+build.3</version>
                <version>1.20.5-pre1+build.4</version>
                <version>1.20.5-pre1+build.5</version>
                <version>1.20.5-pre2+build.1</version>
                <version>1.20.5-pre2+build.2</version>
                <version>1.20.5-pre3+build.1</version>
                <version>1.20.5-pre3+build.2</version>
                <version>1.20.5-pre3+build.3</version>
                <version>1.20.5-pre4+build.1</version>
                <version>1.20.5-rc1+build.1</version>
                <version>1.20.5-rc1+build.2</version>
                <version>1.20.5-rc1+build.3</version>
                <version>1.20.5-rc2+build.1</version>
                <version>1.20.5-rc2+build.2</version>
                <version>1.20.5-rc3+build.1</version>
                <version>1.20.5-rc3+build.2</version>
                <version>1.20.5+build.1</version>
                <version>1.20.6-rc1+build.1</version>
                <version>1.20.6-rc1+build.2</version>
                <version>1.20.6-rc1+build.3</version>
                <version>1.20.6-rc1+build.4</version>
                <version>1.20.6+build.1</version>
                <version>24w18a+build.1</version>
                <version>24w18a+build.2</version>
                <version>24w18a+build.3</version>
                <version>24w18a+build.4</version>
                <version>24w19a+build.1</version>
                <version>24w19a+build.2</version>
                <version>24w19b+build.1</version>
                <version>24w19b+build.2</version>
                <version>24w19b+build.3</version>
                <version>24w19b+build.4</version>
                <version>24w20a+build.2</version>
                <version>24w20a+build.3</version>
                <version>24w20a+build.4</version>
                <version>24w20a+build.5</version>
                <version>1.20.6+build.2</version>
                <version>24w21a+build.1</version>
                <version>24w21b+build.1</version>
                <version>24w21b+build.2</version>
                <version>24w21b+build.3</version>
                <version>1.20.6+build.3</version>
                <version>24w21b+build.4</version>
                <version>24w21b+build.5</version>
                <version>24w21b+build.6</version>
                <version>24w21b+build.7</version>
                <version>24w21b+build.8</version>
                <version>1.21-pre1+build.2</version>
                <version>1.21-pre1+build.3</version>
                <version>1.21-pre1+build.4</version>
                <version>1.21-pre1+build.5</version>
                <version>1.21-pre2+build.1</version>
                <version>1.21-pre2+build.2</version>
                <version>1.21-pre3+build.1</version>
                <version>1.21-pre4+build.1</version>
                <version>1.21-pre4+build.2</version>
            </versions>
        </versioning>
    </metadata>";

    let manifest: YarnManifest = quick_xml::de::from_str(manifest).unwrap();

    let v1_21_pre4 = MinecraftVersion::new_pre_release(1, 21, 0, 4).unwrap();
    assert_eq!(manifest.get_versions(&v1_21_pre4).len(), 2);

    assert!(v1_21_pre4.is_same(&manifest.versions.latest.split().0));
    assert!(v1_21_pre4.is_same(&manifest.versions.release.split().0));

    let v1_20_5_rc1 = MinecraftVersion::new_pre_release(1, 20, 5, 1).unwrap();
    assert_eq!(manifest.get_versions(&v1_20_5_rc1).len(), 4);

    let v1_20_5_rc2 = MinecraftVersion::new_pre_release(1, 20, 5, 2).unwrap();
    assert_eq!(manifest.get_versions(&v1_20_5_rc2).len(), 2);

    let v1_20_6 = MinecraftVersion::new_release(1, 20, 6);
    assert_eq!(manifest.get_versions(&v1_20_6).len(), 3);

    let v1_21_pre1 = MinecraftVersion::new_pre_release(1, 21, 0, 1).unwrap();
    assert_eq!(manifest.get_versions(&v1_21_pre1).len(), 4);

    let v1_21_pre2 = MinecraftVersion::new_pre_release(1, 21, 0, 2).unwrap();
    assert_eq!(manifest.get_versions(&v1_21_pre2).len(), 2);

    let v1_21_pre3 = MinecraftVersion::new_pre_release(1, 21, 0, 3).unwrap();
    assert_eq!(manifest.get_versions(&v1_21_pre3).len(), 1);
}
