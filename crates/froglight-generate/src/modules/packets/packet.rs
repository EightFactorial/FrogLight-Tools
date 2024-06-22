use std::{io::SeekFrom, path::Path};

use froglight_extract::bundle::ExtractBundle;
use proc_macro2::{Span, TokenStream};
use serde_json::Value;
use syn::{
    punctuated::Punctuated,
    token::{Brace, Pub, Semi, Struct},
    Attribute, Field, FieldMutability, Fields, FieldsNamed, File, Generics, Ident, Item,
    ItemStruct, Type, Visibility,
};
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use tracing::trace;

use super::Packets;
use crate::{bundle::GenerateBundle, helpers::update_file_tag};

impl Packets {
    pub(super) async fn create_packet(
        packet_name: &str,
        module_name: &str,
        packet_data: &Value,
        path: &Path,

        _generate: &GenerateBundle<'_>,
        _extract: &ExtractBundle<'_>,
    ) -> anyhow::Result<()> {
        let mut packet_path = path.join(module_name);
        packet_path.set_extension("rs");

        let mut packet_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&packet_path)
            .await?;

        // Read the contents of the packet file
        let mut contents = String::new();
        packet_file.read_to_string(&mut contents).await?;

        // Skip recreating the packet if it is missing the `@generated` tag
        if contents.is_empty() || contents.contains("@generated") {
            trace!("Creating packet at \"{}\"", packet_path.display());

            let output = Self::create_packet_inner(packet_name, packet_data);

            packet_file.seek(SeekFrom::Start(0)).await?;
            packet_file.set_len(0).await?;
            packet_file.write_all(output.as_bytes()).await?;

            update_file_tag(&mut packet_file, &packet_path).await
        } else {
            trace!("Skipping packet at \"{}\"", packet_path.display());
            Ok(())
        }
    }

    fn create_packet_inner(packet_name: &str, packet_data: &Value) -> String {
        let mut packet_file = File { shebang: None, attrs: Vec::new(), items: Vec::new() };

        let fields = packet_data["fields"].as_array().unwrap();
        let fields: Vec<&str> = fields.iter().map(|f| f.as_str().unwrap()).collect();

        // Add imports
        let imports = Self::imports_from_fields(&fields);
        packet_file.items.extend(imports);
        packet_file.items.push(Item::Verbatim(TokenStream::new()));

        // Add packet struct
        let packet = if fields.is_empty() {
            // If the packet has no fields, return a unit struct
            ItemStruct {
                attrs: Self::attrs_from_fields(&fields),
                vis: Visibility::Public(Pub::default()),
                struct_token: Struct::default(),
                ident: Ident::new(packet_name, Span::call_site()),
                generics: Generics::default(),
                semi_token: Some(Semi::default()),
                fields: Fields::Unit,
            }
        } else {
            // Create the packet struct
            ItemStruct {
                attrs: Self::attrs_from_fields(&fields),
                vis: Visibility::Public(Pub::default()),
                struct_token: Struct::default(),
                ident: Ident::new(packet_name, Span::call_site()),
                generics: Generics::default(),
                semi_token: None,
                fields: Self::packet_fields(&fields),
            }
        };
        packet_file.items.push(Item::Struct(packet));

        // Return the formatted file
        prettyplease::unparse(&packet_file)
    }

    /// Import the required modules for the packet struct
    fn imports_from_fields(fields: &[&str]) -> Vec<Item> {
        let mut imports = Vec::new();
        imports.push(syn::parse_quote! { use froglight_macros::FrogReadWrite; });

        if fields.iter().any(|&f| f == "BlockPos") {
            imports.push(syn::parse_quote! { use crate::common::BlockPosition; });
        }

        if fields.iter().any(|&f| f == "GameProfile") {
            imports.push(syn::parse_quote! { use crate::common::GameProfile; });
        }

        if fields.iter().any(|&f| f == "HashMap") {
            imports.push(syn::parse_quote! {
                #[cfg(not(feature = "hashbrown"))]
                use std::collections::HashMap;
            });
            imports.push(syn::parse_quote! {
                #[cfg(feature = "hashbrown")]
                use hashbrown::HashMap;
            });
        }

        if fields.iter().any(|&f| f == "Json") {
            imports.push(syn::parse_quote! { use serde::{Serialize, Deserialize}; });
        }

        if fields.iter().any(|&f| f == "Nbt") {
            imports.push(syn::parse_quote! { use simdnbt::owned::Nbt; });
        }

        if fields.iter().any(|&f| f == "ResourceLocation") {
            imports.push(syn::parse_quote! { use crate::common::ResourceKey; });
        }

        if fields.iter().any(|&f| f == "String") {
            imports.push(syn::parse_quote! { use compact_str::CompactString; });
        }

        if fields.iter().any(|&f| f == "Uuid") {
            imports.push(syn::parse_quote! { use uuid::Uuid; });
        }

        if fields.len() == 1 {
            imports.push(syn::parse_quote! { use derive_more::{Deref, DerefMut, From, Into}; });
        }

        imports.into_iter().map(Item::Use).collect()
    }

    /// Create the attributes for the packet struct
    fn attrs_from_fields(fields: &[&str]) -> Vec<Attribute> {
        let mut attrs = Vec::new();

        {
            let mut derives = TokenStream::new();

            // Always derive `Debug`, `Clone`, and `PartialEq`
            derives.extend(quote::quote! { Debug, Clone, });

            // If the packet doesn't have any Vecs, HashMaps, Strings, or ResourceLocations,
            // derive `Copy`
            if fields.iter().all(|&f| {
                !f.contains("Vec")
                    && !f.contains("HashMap")
                    && !matches!(f, "GameProfile" | "Nbt" | "String" | "ResourceLocation")
            }) {
                derives.extend(quote::quote! { Copy, });
            }

            // Always derive `PartialEq`
            derives.extend(quote::quote! { PartialEq, });

            // If the packet doesn't have any floats, derive `Eq` and `Hash`
            if fields.iter().all(|&f| !matches!(f, "f32" | "f64" | "Nbt")) {
                derives.extend(quote::quote! { Eq, });

                // If the packet doesn't have any HashMaps, derive `Hash`
                if fields.iter().all(|&f| !matches!(f, "GameProfile" | "HashMap")) {
                    derives.extend(quote::quote! { Hash, });
                }
            }

            // If the packet only has one field, derive `Deref`, `DerefMut`,
            // `From`, and `Into`
            if fields.len() == 1 {
                derives.extend(quote::quote! { Deref, DerefMut, From, Into, });
            }

            // If the packet is a unit struct, derive `Default`
            if fields.is_empty() {
                derives.extend(quote::quote! { Default, });
            }

            // If the packet is JSON, derive `Serialize` and `Deserialize`
            if fields.iter().any(|&f| f == "Json") {
                derives.extend(quote::quote! { Serialize, Deserialize, });
            }

            // Always derive `FrogReadWrite`
            derives.extend(quote::quote! { FrogReadWrite });

            attrs.push(syn::parse_quote! { #[derive(#derives)] });
        }

        // Always derive `Reflect` if the `bevy` feature is enabled
        attrs.push(
            syn::parse_quote! { #[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))] },
        );

        // Mark the struct to be ser/de as JSON
        if fields.iter().any(|&f| f == "Json") {
            attrs.push(syn::parse_quote! { #[frog(json)] });
        }

        // If the struct has no fields give it empty tests
        if fields.is_empty() {
            attrs.push(syn::parse_quote! {
                #[frog(tests = ["read_verify", "write_verify"], bytes = [])]
            });
        }

        attrs
    }

    /// Create the fields for the packet struct
    fn packet_fields(packet_data: &[&str]) -> Fields {
        let mut named = Punctuated::new();

        for (index, &field) in packet_data.iter().enumerate() {
            let mut value = field.to_string();

            let mut is_var = false;
            if value.starts_with("Var") {
                is_var = true;
            }

            // Replace the extracted field type with the correct type
            value = match value.as_str() {
                "BlockPos" => String::from("BlockPosition"),
                "HashMap" => String::from("HashMap<(), ()>"),
                "Option" => String::from("Option<()>"),
                "ResourceLocation" => String::from("ResourceKey"),
                "String" => String::from("CompactString"),
                "Vec" => String::from("Vec<()>"),
                "VarInt" => String::from("u32"),
                "VarLong" => String::from("u64"),
                _ => value,
            };

            named.push(Field {
                attrs: if is_var { vec![syn::parse_quote! { #[frog(var)] }] } else { Vec::new() },
                vis: Visibility::Public(Pub::default()),
                mutability: FieldMutability::None,
                ident: Some(Ident::new(&format!("field_{index}"), Span::call_site())),
                colon_token: None,
                ty: syn::parse_str::<Type>(&value).unwrap(),
            });
        }

        Fields::Named(FieldsNamed { brace_token: Brace::default(), named })
    }
}
