use convert_case::{Case, Casing};
use froglight_generate::{CliArgs, DataMap};
use hashbrown::{HashMap, HashSet};

pub(super) const METADATA_ACTIONS: &[(&str, MetadataAction)] = &[
    // ("air_supply", MetadataAction::Type("(u32)", None)),
    ("air_supply", MetadataAction::Type("()", None)),
    ("armadillo_state", MetadataAction::Type("(u32)", Some("0u32"))),
    ("arrow_count", MetadataAction::Type("(u32)", Some("0u32"))),
    ("baby", MetadataAction::Type("(bool)", Some("false"))),
    ("biting", MetadataAction::Type("(bool)", Some("false"))),
    ("boost_time", MetadataAction::Type("(u32)", Some("0u32"))),
    ("brightness_override", MetadataAction::Type("(bool)", Some("false"))),
    ("bubble_time", MetadataAction::Type("(u32)", Some("0u32"))),
    ("can_duplicate", MetadataAction::Type("(bool)", Some("true"))),
    ("client_anger_level", MetadataAction::Type("(u32)", Some("0u32"))),
    ("converting", MetadataAction::Type("(bool)", Some("false"))),
    ("custom_name_visible", MetadataAction::Type("(bool)", Some("true"))),
    ("custom_name", MetadataAction::Type("(CompactString)", None)),
    ("dancing", MetadataAction::Type("(bool)", Some("false"))),
    ("dangerous", MetadataAction::Type("(bool)", Some("false"))),
    ("dark_ticks_remaining", MetadataAction::Type("(u32)", Some("0u32"))),
    ("dash", MetadataAction::Type("(bool)", Some("false"))),
    ("display_block", MetadataAction::Type("(u32)", Some("9u32"))),
    ("drop_seed_at_tick", MetadataAction::Type("(u32)", Some("0u32"))),
    ("eat_counter", MetadataAction::Type("(u32)", Some("0u32"))),
    ("foil", MetadataAction::Type("(bool)", Some("false"))),
    ("from_bucket", MetadataAction::Type("(bool)", Some("false"))),
    ("fuel", MetadataAction::Type("(bool)", Some("false"))),
    ("fuse", MetadataAction::Type("(u32)", Some("40u32"))),
    ("going_home", MetadataAction::Type("(bool)", Some("false"))),
    ("got_fish", MetadataAction::Type("(bool)", Some("false"))),
    ("has_egg", MetadataAction::Type("(bool)", Some("false"))),
    ("has_left_horn", MetadataAction::Type("(bool)", Some("true"))),
    ("has_right_horn", MetadataAction::Type("(bool)", Some("true"))),
    // ("health", MetadataAction::Type("(f32)", None)),
    ("health", MetadataAction::Type("()", None)),
    ("height", MetadataAction::Type("(f32)", Some("1f32"))),
    ("hurt", MetadataAction::Type("(bool)", Some("false"))),
    ("hurtdir", MetadataAction::Remove),
    ("immune_to_zombification", MetadataAction::Type("(bool)", Some("false"))),
    ("interested", MetadataAction::Type("(bool)", Some("false"))),
    ("is_celebrating", MetadataAction::Type("(bool)", Some("false"))),
    ("is_charging_crossbow", MetadataAction::Type("(bool)", Some("false"))),
    ("is_charging", MetadataAction::Type("(bool)", Some("false"))),
    ("is_dancing", MetadataAction::Type("(bool)", Some("false"))),
    ("is_ignited", MetadataAction::Type("(bool)", Some("false"))),
    ("is_lying", MetadataAction::Type("(bool)", Some("false"))),
    ("is_powered", MetadataAction::Type("(bool)", Some("false"))),
    ("is_screaming_goat", MetadataAction::Type("(bool)", Some("false"))),
    ("laying_egg", MetadataAction::Type("(bool)", Some("false"))),
    ("left_rotation", MetadataAction::Type("(f32)", Some("0f32"))),
    ("loyalty", MetadataAction::Type("(bool)", Some("false"))),
    ("moistness_level", MetadataAction::Type("(u32)", Some("0u32"))),
    ("moving", MetadataAction::Type("(bool)", Some("false"))),
    ("no_gravity", MetadataAction::Type("(bool)", Some("false"))),
    ("owneruuid", MetadataAction::NameAndType("OwnerUuid", "(Uuid)", None)),
    ("paddle_left", MetadataAction::Type("(bool)", Some("false"))),
    ("paddle_right", MetadataAction::Type("(bool)", Some("false"))),
    ("painting_variant", MetadataAction::Type("(u32)", Some("0u32"))),
    ("peek", MetadataAction::Type("(bool)", Some("false"))),
    ("phase", MetadataAction::Type("(u32)", Some("0u32"))),
    ("pierce_level", MetadataAction::Type("(u32)", Some("0u32"))),
    ("player_absorption", MetadataAction::Type("(u32)", Some("0u32"))),
    ("player_main_hand", MetadataAction::Type("(bool)", Some("true"))),
    ("playing_dead", MetadataAction::Type("(bool)", Some("false"))),
    ("puff_state", MetadataAction::Type("(u32)", Some("0u32"))),
    ("pumpkin", MetadataAction::Type("(bool)", Some("false"))),
    ("radius", MetadataAction::Type("(f32)", Some("1f32"))),
    ("relax_state_one", MetadataAction::Type("(bool)", Some("false"))),
    ("remaining_anger_time", MetadataAction::Type("(u32)", Some("0u32"))),
    ("right_rotation", MetadataAction::Type("(f32)", Some("0f32"))),
    ("rotation", MetadataAction::Type("(f32)", Some("0f32"))),
    ("saddle", MetadataAction::Type("(bool)", Some("false"))),
    ("scale", MetadataAction::Type("(f32)", Some("1f32"))),
    ("score", MetadataAction::Type("(u32)", Some("0u32"))),
    // ("shadow_radius", MetadataAction::Type("(f32)", None)),
    ("shadow_radius", MetadataAction::Type("()", None)),
    // ("shadow_strength", MetadataAction::Type("(f32)", None)),
    ("shadow_strength", MetadataAction::Type("()", None)),
    ("sheared", MetadataAction::Type("(bool)", Some("false"))),
    ("shoulder_left", MetadataAction::Remove),
    ("shoulder_right", MetadataAction::Remove),
    ("show_bottom", MetadataAction::Type("(bool)", Some("true"))),
    ("silent", MetadataAction::Type("(bool)", Some("false"))),
    ("size", MetadataAction::Type("(f32)", Some("1f32"))),
    ("sneeze_counter", MetadataAction::Type("(u32)", Some("0u32"))),
    ("special_type", MetadataAction::Type("(bool)", Some("false"))),
    ("spell_casting", MetadataAction::Type("(bool)", Some("false"))),
    ("standing", MetadataAction::Type("(bool)", Some("true"))),
    ("stared_at", MetadataAction::Type("(bool)", Some("false"))),
    ("state", MetadataAction::Type("(u32)", Some("0u32"))),
    ("stinger_count", MetadataAction::Type("(u32)", Some("0u32"))),
    // ("strength", MetadataAction::Type("(f32)", None)),
    ("strength", MetadataAction::Type("()", None)),
    ("suffocating", MetadataAction::Type("(bool)", Some("false"))),
    ("swell_dir", MetadataAction::Remove),
    ("ticks_frozen", MetadataAction::Type("(u32)", Some("0u32"))),
    ("travelling", MetadataAction::Type("(bool)", Some("false"))),
    ("trusting", MetadataAction::Type("(bool)", Some("false"))),
    ("type_variant", MetadataAction::Type("(u32)", Some("0u32"))),
    ("type", MetadataAction::Type("(u32)", Some("0u32"))),
    ("unhappy_counter", MetadataAction::Type("(u32)", Some("0u32"))),
    ("using_item", MetadataAction::Type("(bool)", Some("false"))),
    ("variant", MetadataAction::Type("(u32)", Some("0u32"))),
    // ("view_range", MetadataAction::Type("(u32)", None)),
    ("view_range", MetadataAction::Type("()", None)),
    ("waiting", MetadataAction::Type("(bool)", Some("false"))),
    ("width", MetadataAction::Type("(f32)", Some("1f32"))),
    ("wool", MetadataAction::Type("(u32)", Some("0u32"))),
];

pub(super) enum MetadataAction {
    NameAndType(&'static str, &'static str, Option<&'static str>),
    Type(&'static str, Option<&'static str>),
    Remove,
}

pub(super) async fn generate_metadata(datamap: &DataMap, args: &CliArgs) -> anyhow::Result<()> {
    // Sort the versions using the manifest.
    let mut versions: Vec<_> = datamap.version_data.keys().collect();
    versions.sort_by(|a, b| datamap.manifest.compare(a, b).unwrap());

    // Collect the latest data for all entities.
    let mut entities = HashMap::new();
    for data in datamap.version_data.values() {
        for entity in data.entities.iter() {
            entities.insert(&entity.name, entity);
        }
    }

    let mut metadata_content = String::new();

    // Get all of the metadata keys.
    let mut metadata = HashSet::new();
    let mut cat_and_type = HashSet::new();

    for entity in entities.values() {
        metadata.extend(entity.metadata.iter());

        cat_and_type.insert(entity.category.to_string());
        cat_and_type.insert(format!("{}Entity", entity.kind));
    }

    // Sort the categories and types and insert them
    metadata_content.push_str("    // Mob Categories and Types\n");
    let mut cat_and_type = cat_and_type.into_iter().collect::<Vec<_>>();
    cat_and_type.sort_by_key(|a| a.to_ascii_lowercase());

    for cat_or_type in cat_and_type {
        metadata_content.push_str(&format!("    {},\n", cat_or_type.to_case(Case::Pascal)));
    }

    // Sort the metadata by name.
    metadata_content.push_str("    // Entity Components\n");
    let mut metadata = metadata.into_iter().collect::<Vec<_>>();
    metadata.sort();

    for (index, meta) in metadata.iter().enumerate() {
        let ident = meta.to_case(Case::Pascal);

        if let Some((_, action)) = METADATA_ACTIONS.iter().find(|(key, _)| key == meta) {
            match action {
                MetadataAction::NameAndType(ident, data, default) => {
                    if let Some(default) = default {
                        metadata_content.push_str(&format!("    {ident} => {data} = {default}"));
                    } else {
                        metadata_content.push_str(&format!("    {ident} => {data}"));
                    }
                }
                MetadataAction::Type(data, default) => {
                    if let Some(default) = default {
                        metadata_content.push_str(&format!("    {ident} => {data} = {default}"));
                    } else {
                        metadata_content.push_str(&format!("    {ident} => {data}"));
                    }
                }
                MetadataAction::Remove => continue,
            }

            if index < metadata.len() - 1 {
                metadata_content.push_str(",\n");
            }
        } else {
            tracing::warn!("EntityGenerator: Metadata type not found for \"{ident}\"");

            if index < metadata.len() - 1 {
                metadata_content.push_str(&format!("    {ident} => (), // TODO\n"));
            } else {
                metadata_content.push_str(&format!("    {ident} => () // TODO"));
            }
        }
    }

    let content = format!(
        r#"//! Generated entity components.
//!
//! @generated by 'TODO'
#![allow(clippy::unreadable_literal, clippy::wildcard_imports, missing_docs, unused_parens)]

use bevy_ecs::component::Component;
#[cfg(feature = "reflect")]
use bevy_ecs::reflect::ReflectComponent;
#[cfg(feature = "reflect")]
use bevy_reflect::Reflect;
use compact_str::CompactString;
use uuid::Uuid;

froglight_macros::impl_generated_components! {{
{metadata_content}
}}
"#
    );

    let file_path = args.dir.join("crates/froglight-entity/src/generated/component.rs");
    if !file_path.exists() {
        tracing::warn!("EntityGenerator: Creating file \"{}\"", file_path.display());
        tokio::fs::create_dir_all(file_path.parent().unwrap()).await?;
    }
    tokio::fs::write(file_path, &content).await?;

    Ok(())
}
