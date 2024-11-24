use convert_case::{Case, Casing};
use froglight_generate::{CliArgs, DataMap};
use hashbrown::{HashMap, HashSet};

pub(super) const METADATA_ACTIONS: &[(&str, MetadataAction)] = &[
    ("air_supply", MetadataAction::Type("(u32)")),
    ("armadillo_state", MetadataAction::Type("(u32)")),
    ("baby", MetadataAction::Type("(bool)")),
    ("biting", MetadataAction::Type("(bool)")),
    ("boost_time", MetadataAction::Type("(u32)")),
    ("brightness_override", MetadataAction::Type("(u32)")),
    ("bubble_time", MetadataAction::Type("(u32)")),
    ("can_duplicate", MetadataAction::Type("(bool)")),
    ("client_anger_level", MetadataAction::Type("(u32)")),
    ("converting", MetadataAction::Type("(bool)")),
    ("custom_name_visible", MetadataAction::Type("(bool)")),
    ("custom_name", MetadataAction::Type("(CompactString)")),
    ("dancing", MetadataAction::Type("(bool)")),
    ("dangerous", MetadataAction::Type("(bool)")),
    ("dark_ticks_remaining", MetadataAction::Type("(u32)")),
    ("dash", MetadataAction::Type("(bool)")),
    ("display_block", MetadataAction::Type("(u32)")),
    ("drop_seed_at_tick", MetadataAction::Type("(u32)")),
    ("eat_counter", MetadataAction::Type("(u32)")),
    ("from_bucket", MetadataAction::Type("(bool)")),
    ("fuel", MetadataAction::Type("(bool)")),
    ("fuse", MetadataAction::Type("(f32)")),
    ("going_home", MetadataAction::Type("(bool)")),
    ("got_fish", MetadataAction::Type("(bool)")),
    ("has_egg", MetadataAction::Type("(bool)")),
    ("has_left_horn", MetadataAction::Type("(bool)")),
    ("has_right_horn", MetadataAction::Type("(bool)")),
    ("health", MetadataAction::Type("(f32)")),
    ("height", MetadataAction::Type("(f32)")),
    ("hurt", MetadataAction::Type("(bool)")),
    ("hurtdir", MetadataAction::Remove),
    ("immune_to_zombification", MetadataAction::Type("(bool)")),
    ("interested", MetadataAction::Type("(bool)")),
    ("is_celebrating", MetadataAction::Type("(bool)")),
    ("is_charging_crossbow", MetadataAction::Type("(bool)")),
    ("is_charging", MetadataAction::Type("(bool)")),
    ("is_dancing", MetadataAction::Type("(bool)")),
    ("is_ignited", MetadataAction::Type("(bool)")),
    ("is_lying", MetadataAction::Type("(bool)")),
    ("is_powered", MetadataAction::Type("(bool)")),
    ("is_screaming_goat", MetadataAction::Type("(bool)")),
    ("laying_egg", MetadataAction::Type("(bool)")),
    ("left_rotation", MetadataAction::Type("(f32)")),
    ("loyalty", MetadataAction::Type("(bool)")),
    ("moistness_level", MetadataAction::Type("(u32)")),
    ("moving", MetadataAction::Type("(bool)")),
    ("no_gravity", MetadataAction::Type("(bool)")),
    ("owneruuid", MetadataAction::NameAndType("OwnerUuid", "(Uuid)")),
    ("paddle_left", MetadataAction::Type("(bool)")),
    ("paddle_right", MetadataAction::Type("(bool)")),
    ("painting_variant", MetadataAction::Type("(u32)")),
    ("peek", MetadataAction::Type("(bool)")),
    ("phase", MetadataAction::Type("(u32)")),
    ("pierce_level", MetadataAction::Type("(u32)")),
    ("player_absorption", MetadataAction::Type("(u32)")),
    ("player_main_hand", MetadataAction::Type("(bool)")),
    ("playing_dead", MetadataAction::Type("(bool)")),
    ("puff_state", MetadataAction::Type("(u32)")),
    ("pumpkin", MetadataAction::Type("(bool)")),
    ("radius", MetadataAction::Type("(f32)")),
    ("relax_state_one", MetadataAction::Type("(bool)")),
    ("remaining_anger_time", MetadataAction::Type("(u32)")),
    ("right_rotation", MetadataAction::Type("(f32)")),
    ("rotation", MetadataAction::Type("(f32)")),
    ("saddle", MetadataAction::Type("(bool)")),
    ("scale", MetadataAction::Type("(f32)")),
    ("score", MetadataAction::Type("(u32)")),
    ("shadow_radius", MetadataAction::Type("(f32)")),
    ("shadow_strength", MetadataAction::Type("(f32)")),
    ("sheared", MetadataAction::Type("(bool)")),
    ("shoulder_left", MetadataAction::Type("(bool)")),
    ("shoulder_right", MetadataAction::Type("(bool)")),
    ("show_bottom", MetadataAction::Type("(bool)")),
    ("silent", MetadataAction::Type("(bool)")),
    ("size", MetadataAction::Type("(f32)")),
    ("sneeze_counter", MetadataAction::Type("(u32)")),
    ("special_type", MetadataAction::Type("(u32)")),
    ("spell_casting", MetadataAction::Type("(bool)")),
    ("standing", MetadataAction::Type("(bool)")),
    ("stared_at", MetadataAction::Type("(bool)")),
    ("state", MetadataAction::Type("(u32)")),
    ("strength", MetadataAction::Type("(f32)")),
    ("suffocating", MetadataAction::Type("(bool)")),
    ("swell_dir", MetadataAction::Remove),
    ("ticks_frozen", MetadataAction::Type("(u32)")),
    ("travelling", MetadataAction::Type("(bool)")),
    ("trusting", MetadataAction::Type("(bool)")),
    ("type_variant", MetadataAction::Type("(u32)")),
    ("type", MetadataAction::Type("(u32)")),
    ("unhappy_counter", MetadataAction::Type("(u32)")),
    ("using_item", MetadataAction::Type("(bool)")),
    ("variant", MetadataAction::Type("(u32)")),
    ("view_range", MetadataAction::Type("(u32)")),
    ("waiting", MetadataAction::Type("(bool)")),
    ("width", MetadataAction::Type("(f32)")),
    ("wool", MetadataAction::Type("(u32)")),
];

pub(super) enum MetadataAction {
    NameAndType(&'static str, &'static str),
    Type(&'static str),
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

        cat_and_type.insert(&entity.category);
        cat_and_type.insert(&entity.kind);
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
                MetadataAction::NameAndType(ident, data) => {
                    metadata_content.push_str(&format!("    {ident} => {data}"));
                }
                MetadataAction::Type(data) => {
                    metadata_content.push_str(&format!("    {ident} => {data}"));
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
