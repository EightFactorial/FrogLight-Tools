use convert_case::{Case, Casing};
use froglight_definitions::MinecraftVersion;
use proc_macro2::Span;
use syn::Ident;

/// Generate the struct name.
pub(crate) fn version_struct_name(version: &MinecraftVersion) -> Ident {
    Ident::new(&format!("V{version}").replace('.', "_"), Span::call_site())
}

/// Generate the module name.
pub(crate) fn version_module_name(version: &MinecraftVersion) -> Ident {
    Ident::new(&format!("v{version}").replace('.', "_"), Span::call_site())
}

#[test]
fn test_struct_name() {
    let v1_20_0 = MinecraftVersion::new_release(1, 20, 0);
    assert_eq!(version_struct_name(&v1_20_0).to_string(), "V1_20_0");

    let v1_20_1 = MinecraftVersion::new_release(1, 20, 1);
    assert_eq!(version_struct_name(&v1_20_1).to_string(), "V1_20_1");

    let v1_21_0 = MinecraftVersion::new_release(1, 21, 0);
    assert_eq!(version_struct_name(&v1_21_0).to_string(), "V1_21_0");

    let v1_21_1 = MinecraftVersion::new_release(1, 21, 1);
    assert_eq!(version_struct_name(&v1_21_1).to_string(), "V1_21_1");
}

#[test]
fn test_module_name() {
    let v1_20_0 = MinecraftVersion::new_release(1, 20, 0);
    assert_eq!(version_module_name(&v1_20_0).to_string(), "v1_20_0");

    let v1_20_1 = MinecraftVersion::new_release(1, 20, 1);
    assert_eq!(version_module_name(&v1_20_1).to_string(), "v1_20_1");

    let v1_21_0 = MinecraftVersion::new_release(1, 21, 0);
    assert_eq!(version_module_name(&v1_21_0).to_string(), "v1_21_0");

    let v1_21_1 = MinecraftVersion::new_release(1, 21, 1);
    assert_eq!(version_module_name(&v1_21_1).to_string(), "v1_21_1");
}

/// Convert from a Java `class name` to a Rust `module name`.
pub(crate) fn class_to_module_name(class: &str) -> String {
    let mut class = class.split('/').last().unwrap().to_string();

    if let Some((mut packet, kind)) = class.split_once('$') {
        packet = packet.trim_end_matches("S2CPacket").trim_end_matches("C2SPacket");

        class = format!("{packet}_{kind}")
            .trim_end_matches("S2CPacket")
            .trim_end_matches("C2SPacket")
            .to_string();
    } else {
        class = class.trim_end_matches("S2CPacket").trim_end_matches("C2SPacket").to_string();
    }

    class.to_case(Case::Snake)
}

/// Convert from a Java `class name` to a Rust `struct name`.
pub(crate) fn class_to_struct_name(class: &str) -> String {
    let mut class = class.split('/').last().unwrap().to_string();

    if let Some((mut packet, kind)) = class.split_once('$') {
        packet = packet.trim_end_matches("S2CPacket").trim_end_matches("C2SPacket");

        class = format!("{packet}_{kind}")
            .trim_end_matches("S2CPacket")
            .trim_end_matches("C2SPacket")
            .to_string();
    } else {
        class = class.trim_end_matches("S2CPacket").trim_end_matches("C2SPacket").to_string();
    }

    format!("{class}Packet").to_case(Case::Pascal)
}
