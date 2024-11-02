use reqwest::Client;

use crate::{
    file::{
        protocol::{ArrayArgs, BufferArgs, ProtocolType, ProtocolTypeArgs, ProtocolTypeMap},
        DataPath, FileTrait, VersionProtocol,
    },
    Version,
};

/// All native types, as of v1.21.1
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
    let client = Client::new();
    let cache = crate::file::target_dir().await;

    let v1_20_6 = Version::new_release(1, 20, 6);
    let v1_21_0 = Version::new_release(1, 21, 0);
    let v1_21_1 = Version::new_release(1, 21, 1);
    let datapaths = DataPath::fetch(&v1_21_0, &cache, &(), false, &client).await.unwrap();

    // Fetch the protocols for multiple versions.
    let p1_20_6 =
        VersionProtocol::fetch(&v1_20_6, &cache, &datapaths, false, &client).await.unwrap();
    let p1_21_0 =
        VersionProtocol::fetch(&v1_21_0, &cache, &datapaths, false, &client).await.unwrap();
    let p1_21_1 =
        VersionProtocol::fetch(&v1_21_1, &cache, &datapaths, false, &client).await.unwrap();

    for protocol in [&p1_20_6, &p1_21_0, &p1_21_1] {
        // Check that serialization and deserialization works.
        let serialized = serde_json::to_string(protocol).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();
        assert_eq!(protocol, &deserialized);

        // Check that all native types are known.
        for (name, data) in protocol.types.iter() {
            if ProtocolType::Named("native".into()) == *data {
                assert!(NATIVE_TYPES.contains(&name.as_str()), "Unknown native type: \"{name}\"");
            }
        }

        // Check that all types are valid.
        for data in protocol.types.values() {
            assert_valid_type(data, &protocol.types);
        }
    }

    // Assert that v1.20.6 and v1.21.0 use different types.
    assert_ne!(p1_20_6.types, p1_21_0.types);

    // Assert that they send the same handshake, status, and login packets.
    assert_eq!(p1_20_6.packets["handshaking"], p1_21_0.packets["handshaking"]);
    assert_eq!(p1_20_6.packets["status"], p1_21_0.packets["status"]);
    assert_eq!(p1_20_6.packets["login"], p1_21_0.packets["login"]);

    // Assert that they don't send the same configuration and play packets.
    assert_ne!(p1_20_6.packets["configuration"], p1_21_0.packets["configuration"]);
    assert_ne!(p1_20_6.packets["play"], p1_21_0.packets["play"]);

    // Assert that v1.21.0 and v1.21.1 are completely identical.
    assert_eq!(p1_21_0, p1_21_1);
}

/// Recursively assert that all types are valid,
/// and that all referenced types exist in the [`ProtocolTypeMap`].
fn assert_valid_type(data: &ProtocolType, types: &ProtocolTypeMap) {
    match data {
        ProtocolType::Named(type_name) => {
            if type_name != "native" {
                assert!(
                    types.contains_key(type_name),
                    "Unknown named protocol type: \"{type_name}\""
                );
            }
        }
        ProtocolType::Inline(type_name, ProtocolTypeArgs::Array(array_args)) => {
            assert_eq!(type_name, "array");
            match array_args {
                ArrayArgs::CountField { kind, .. } => {
                    assert_valid_type(kind, types);
                }
                ArrayArgs::Count { count_type, kind } => {
                    assert!(
                        types.contains_key(count_type),
                        "Unknown array count protocol type: \"{count_type}\"",
                    );
                    assert_valid_type(kind, types);
                }
            }
        }
        ProtocolType::Inline(type_name, ProtocolTypeArgs::Bitfield(..)) => {
            assert_eq!(type_name, "bitfield");
        }
        ProtocolType::Inline(type_name, ProtocolTypeArgs::Buffer(buffer_args)) => {
            assert_eq!(type_name, "buffer");
            if let BufferArgs::CountType(count_type) = buffer_args {
                assert!(
                    types.contains_key(count_type),
                    "Unknown buffer count protocol type: \"{count_type}\"",
                );
            }
        }
        ProtocolType::Inline(type_name, ProtocolTypeArgs::Container(container_args)) => {
            assert_eq!(type_name, "container");
            for container_arg in container_args {
                assert_valid_type(&container_arg.kind, types);
            }
        }
        ProtocolType::Inline(type_name, ProtocolTypeArgs::EntityMetadata(entity_metadata_args)) => {
            assert_eq!(type_name, "entityMetadataLoop");
            assert_valid_type(&entity_metadata_args.kind, types);
        }
        ProtocolType::Inline(type_name, ProtocolTypeArgs::Mapper(mapper_args)) => {
            assert_eq!(type_name, "mapper");
            assert_valid_type(&mapper_args.kind, types);
        }
        ProtocolType::Inline(type_name, ProtocolTypeArgs::Option(option_type)) => {
            assert_eq!(type_name, "option");
            assert_valid_type(option_type, types);
        }
        ProtocolType::Inline(type_name, ProtocolTypeArgs::PString(pstring_args)) => {
            assert_eq!(type_name, "pstring");
            if let BufferArgs::CountType(count_type) = pstring_args {
                assert!(
                    types.contains_key(count_type),
                    "Unknown pstring count protocol type: \"{count_type}\"",
                );
            }
        }
        ProtocolType::Inline(type_name, ProtocolTypeArgs::Switch(switch_args)) => {
            assert_eq!(type_name, "switch");
            for field_type in switch_args.fields.values() {
                assert_valid_type(field_type, types);
            }
        }
        ProtocolType::Inline(type_name, ProtocolTypeArgs::TopBitSetTerminatedArray(array_args)) => {
            assert_eq!(type_name, "topBitSetTerminatedArray");
            assert_valid_type(&array_args.kind, types);
        }
    }
}
