use std::path::{Path, PathBuf};

#[cfg(test)]
use froglight_parse::files::protocol::{DataType, DataTypeArgs, TypesMap};
use froglight_parse::{
    files::{DataPaths, VersionProtocol},
    Version,
};
use reqwest::Client;

use super::FileTrait;

impl FileTrait for VersionProtocol {
    type UrlData = DataPaths;
    fn get_url(version: &Version, data: &Self::UrlData) -> String {
        data.get_java_protocol(version).expect("Version not found")
    }

    fn get_path(version: &Version, cache: &Path) -> PathBuf {
        cache.join(format!("v{version}")).join(Self::FILE_NAME)
    }

    fn fetch(
        version: &Version,
        cache: &Path,
        data: &Self::UrlData,
        redownload: bool,
        client: &Client,
    ) -> impl std::future::Future<Output = anyhow::Result<Self>> + Send + Sync {
        super::fetch_json(version, cache, data, redownload, client)
    }
}

/// All native types, as of v1.21.1
#[cfg(test)]
const NATIVE_TYPES: &[&str] = &[
    "void",
    "bool",
    "u8",
    "u16",
    "u32",
    "u64",
    "i8",
    "i16",
    "i32",
    "i64",
    "f32",
    "f64",
    "option",
    "varint",
    "varlong",
    "pstring",
    "string",
    "UUID",
    "array",
    "arrayWithLengthOffset",
    "bitfield",
    "buffer",
    "restBuffer",
    "anonymousNbt",
    "anonOptionalNbt",
    "container",
    "switch",
    "mapper",
    "entityMetadataLoop",
    "topBitSetTerminatedArray",
];

#[tokio::test]
async fn fetch() {
    // Find the target directory.
    let mut cache = PathBuf::from(env!("CARGO_MANIFEST_DIR")).canonicalize().unwrap();
    while !cache.join("target").exists() {
        assert!(!cache.to_string_lossy().is_empty(), "Couldn't find target directory");
        cache = cache.parent().unwrap().to_path_buf();
    }

    cache.push("target");
    cache.push("froglight-generate");
    tokio::fs::create_dir_all(&cache).await.unwrap();

    let client = Client::new();
    let datapaths = DataPaths::fetch(&Version::new_release(1, 21, 0), &cache, &(), false, &client)
        .await
        .unwrap();

    // Test reading the protocol for multiple versions.
    for version in [Version::new_release(1, 21, 0), Version::new_release(1, 21, 1)] {
        // Fetch the protocol.
        let protocol =
            VersionProtocol::fetch(&version, &cache, &datapaths, false, &client).await.unwrap();

        // Check that all native types are known.
        for (name, data) in protocol.types.iter() {
            if DataType::Named("native".into()) == *data {
                assert!(NATIVE_TYPES.contains(&name.as_str()), "Unknown native type: \"{name}\"");
            }
        }

        // Check that all types have the correct data.
        for data in protocol.types.values() {
            assert_valid_type(data, &protocol.types);
        }
    }
}

#[cfg(test)]
fn assert_valid_type(data: &DataType, types: &TypesMap) {
    match data {
        DataType::Named(type_name) => {
            if type_name != "native" {
                assert!(types.contains_key(type_name), "Unknown named data type: \"{type_name}\"");
            }
        }
        DataType::Inline(data_type, data_args) => match data_type.as_str() {
            "array" => {
                assert!(
                    matches!(data_args, DataTypeArgs::Array(..)),
                    "Array has wrong argument type, got: {data_args:?}"
                );

                if let DataTypeArgs::Array(array_args) = data_args {
                    assert!(types.contains_key(&array_args.count_type), "Unknown Array count type");
                    assert_valid_type(&array_args.kind, types);
                }
            }
            "bitfield" => {
                assert!(
                    matches!(data_args, DataTypeArgs::Bitfield(..)),
                    "Bitfield has wrong argument type, got: {data_args:?}"
                );
            }
            "buffer" => {
                assert!(
                    matches!(data_args, DataTypeArgs::Buffer(..)),
                    "Buffer has wrong argument type, got: {data_args:?}"
                );

                if let DataTypeArgs::Buffer(buffer_args) = data_args {
                    if let Some(count_type) = &buffer_args.count_type {
                        assert!(types.contains_key(count_type), "Unknown Buffer count type");
                    }
                }
            }
            "container" => {
                assert!(
                    matches!(data_args, DataTypeArgs::Container(..)),
                    "Container has wrong argument type, got: {data_args:?}"
                );

                if let DataTypeArgs::Container(container_args) = data_args {
                    for container_arg in container_args {
                        assert_valid_type(&container_arg.kind, types);
                    }
                }
            }
            "entityMetadataLoop" => {
                assert!(
                    matches!(data_args, DataTypeArgs::EntityMetadata(..)),
                    "EntityMetadata has wrong argument type, got: {data_args:?}"
                );

                if let DataTypeArgs::EntityMetadata(entity_metadata_args) = data_args {
                    assert_valid_type(&entity_metadata_args.kind, types);
                }
            }
            "mapper" => {
                assert!(
                    matches!(data_args, DataTypeArgs::Mapper(..)),
                    "Mapper has wrong argument type, got: {data_args:?}"
                );

                if let DataTypeArgs::Mapper(mapper_args) = data_args {
                    assert_valid_type(&mapper_args.kind, types);
                }
            }
            "option" => {
                assert!(
                    matches!(data_args, DataTypeArgs::Option(..)),
                    "Option has wrong argument type, got: {data_args:?}"
                );

                if let DataTypeArgs::Option(option_args) = data_args {
                    assert_valid_type(option_args, types);
                }
            }
            "pstring" => {
                assert!(
                    matches!(data_args, DataTypeArgs::PString(..)),
                    "PString has wrong argument type, got: {data_args:?}"
                );

                if let DataTypeArgs::PString(buffer_args) = data_args {
                    if let Some(count_type) = &buffer_args.count_type {
                        assert!(types.contains_key(count_type), "Unknown PString count type");
                    }
                }
            }
            "switch" => {
                assert!(
                    matches!(data_args, DataTypeArgs::Switch(..)),
                    "Switch has wrong argument type, got: {data_args:?}"
                );

                if let DataTypeArgs::Switch(switch_args) = data_args {
                    for field_type in switch_args.fields.values() {
                        assert_valid_type(field_type, types);
                    }
                }
            }
            _ => panic!("Unknown inline data type: \"{data_type}\""),
        },
    }
}
