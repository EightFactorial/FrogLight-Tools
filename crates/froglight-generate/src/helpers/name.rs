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
