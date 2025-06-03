#![expect(unused_imports)]

use std::{borrow::Cow, path::Path};

use cafebabe::{
    attributes::{AttributeData, AttributeInfo},
    bytecode::Opcode,
    constant_pool::{LiteralConstant, Loadable, MemberRef, ReferenceKind},
    descriptors::{FieldDescriptor, FieldType, ReturnDescriptor},
    ClassFile, MethodInfo,
};
use convert_case::{Case, Casing};
use froglight_dependency::{
    container::DependencyContainer,
    dependency::minecraft::{minecraft_code::CodeBundle, MinecraftCode},
    version::Version,
};
use indexmap::{map::Entry, IndexMap};
use tokio::sync::OnceCell;
use tracing::{debug, error, trace};

use super::Entities;
use crate::{
    class_helper::{ClassHelper, OwnedConstant},
    ToolConfig,
};

const BUILDER_TYPE: &str = "net/minecraft/entity/attribute/DefaultAttributeContainer$Builder";
const BUILDER_DESCRIPTOR: &str =
    "()Lnet/minecraft/entity/attribute/DefaultAttributeContainer$Builder;";

const ENTITY_ATTRIBUTES: &str = "net/minecraft/entity/attribute/EntityAttributes";
const REGISTRY_TYPE_DESCRIPTOR: &str = "Lnet/minecraft/registry/entry/RegistryEntry;";

const ENTITY_TYPE: &str = "net/minecraft/entity/EntityType";
const ENTITY_TYPE_DESCRIPTOR: &str = "Lnet/minecraft/entity/EntityType;";

impl Entities {
    /// Extract the entity attribute values for each entity type.
    ///
    /// # Note
    /// Any entity attributes that do not have defined values
    /// are left with the value set to "DEFAULT".
    pub(super) async fn extract_entity_attribute_values(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<IndexMap<String, IndexMap<String, String>>> {
        let mut attributes = IndexMap::<String, IndexMap<String, String>>::new();

        deps.get_or_retrieve::<MinecraftCode>().await?;
        deps.scoped_fut::<MinecraftCode, anyhow::Result<_>>(async |jars, deps| {
            let jar = jars.get_version(version, deps).await?;
            let class = jar.get(ENTITY_TYPE).ok_or_else(|| {
                anyhow::anyhow!("Packets: Could not find \"{ENTITY_TYPE}\" class!")
            })?;

            let mut classes = Vec::new();

            let initial = class.class_code().bytecode.as_ref().unwrap();
            let initial = initial.opcodes.iter().map(|(_, opcode)| opcode).collect::<Vec<_>>();
            class.iter_code_recursive(&initial, jar, |opcode| match opcode {
                // Initializing any entity classes that are not AI, animation-related, boss-related.
                Opcode::Invokespecial(MemberRef { class_name, name_and_type })
                    if class_name.starts_with("net/minecraft/entity/")
                        && !class_name.starts_with("net/minecraft/entity/ai")
                        && !class_name.starts_with("net/minecraft/entity/boss")
                        && !class_name.contains("$")
                        && class_name != "net/minecraft/entity/AnimationState"
                        && name_and_type.name == "<init>" =>
                {
                    classes.push(class_name.to_string())
                }
                // Note: This is where the class init is called, so we finally get the class name!
                Opcode::Invokedynamic(dynamic) => {
                    for method in class.class_bootstrap_methods(dynamic.attr_index) {
                        if method.kind == ReferenceKind::NewInvokeSpecial {
                            classes.push(method.class_name.to_string());
                        }
                    }
                }
                // Actually creating the `EntityType` here,
                // so stop and process all collected classes so far.
                Opcode::Putstatic(MemberRef { class_name, name_and_type })
                    if class_name == ENTITY_TYPE
                        && name_and_type.descriptor == ENTITY_TYPE_DESCRIPTOR =>
                {
                    let mut collected = IndexMap::new();
                    for class in core::mem::take(&mut classes) {
                        debug!("[{}]: Parsing class \"{class}\"", name_and_type.name);
                        for (attr, value) in Self::parse_class_attributes(&class, jar) {
                            collected.insert(attr.to_case(Case::Pascal), value);
                        }
                    }
                    attributes.insert(name_and_type.name.to_lowercase(), collected);
                }
                Opcode::Putstatic(..) => classes.clear(),
                _ => {}
            });

            Ok(())
        })
        .await?;

        Ok(attributes)
    }

    /// Iterate over all of the methods in the class and
    /// look for an attribute builder method.
    ///
    /// Use this to recursively parse attributes through the class hierarchy.
    fn parse_class_attributes(class_name: &str, jar: &CodeBundle) -> IndexMap<String, String> {
        if class_name.starts_with("net/minecraft") {
            if let Some(class) = jar.get(class_name) {
                if let Some(method) =
                    class.methods.iter().find(|m| m.descriptor.to_string() == BUILDER_DESCRIPTOR)
                {
                    debug!("    [{class_name}]: Using \"{}\" as the entrypoint", method.name);
                    return Self::parse_class_method_attributes(&class, method, jar);
                }
            }
        }

        IndexMap::new()
    }

    /// Recursively parse a class method to find entity attributes.
    fn parse_class_method_attributes(
        class: &ClassFile,
        method: &MethodInfo,
        jar: &CodeBundle,
    ) -> IndexMap<String, String> {
        let mut attributes = IndexMap::new();

        if let Some(AttributeInfo { data: AttributeData::Code(code), .. }) =
            method.attributes.iter().find(|attr| matches!(&attr.data, AttributeData::Code(..)))
        {
            let mut constants = Vec::<OwnedConstant>::new();

            let initial = code.bytecode.as_ref().unwrap();
            let initial = initial.opcodes.iter().map(|(_, opcode)| opcode).collect::<Vec<_>>();
            class.iter_code_recursive(&initial, jar, |opcode| {
                match opcode {
                    // Collect constants used in the method for later use
                    Opcode::Dconst0 => constants.push(OwnedConstant::Double(0.0)),
                    Opcode::Dconst1 => constants.push(OwnedConstant::Double(1.0)),
                    Opcode::Ldc(Loadable::LiteralConstant(constant))
                    | Opcode::LdcW(Loadable::LiteralConstant(constant))
                    | Opcode::Ldc2W(Loadable::LiteralConstant(constant)) => {
                        constants.push(OwnedConstant::from(constant));
                    }
                    // Collect attribute names and store them as constants for later use
                    Opcode::Getstatic(MemberRef { class_name, name_and_type })
                        if class_name == ENTITY_ATTRIBUTES
                            && name_and_type.descriptor == REGISTRY_TYPE_DESCRIPTOR =>
                    {
                        constants.push(OwnedConstant::String(name_and_type.name.to_string()));
                    }
                    // Recurse into other builder methods and collect their attributes
                    Opcode::Invokestatic(MemberRef { class_name, name_and_type })
                    | Opcode::Invokevirtual(MemberRef { class_name, name_and_type })
                    | Opcode::Invokeinterface(MemberRef { class_name, name_and_type }, ..)
                        if name_and_type.descriptor == BUILDER_DESCRIPTOR =>
                    {
                        attributes.extend(
                            Self::find_and_parse_class_method(class_name, &name_and_type.name, jar),
                        );
                    }
                    // Add attributes to the map when added to the builder
                    Opcode::Invokestatic(MemberRef { class_name, name_and_type })
                    | Opcode::Invokevirtual(MemberRef { class_name, name_and_type })
                    | Opcode::Invokeinterface(MemberRef { class_name, name_and_type }, ..)
                        if class_name == BUILDER_TYPE => {
                            match name_and_type.name.as_ref() {
                                "add" if &name_and_type.descriptor == "(Lnet/minecraft/registry/entry/RegistryEntry;)Lnet/minecraft/entity/attribute/DefaultAttributeContainer$Builder;" => {
                                    match constants.pop() {
                                        #[allow(unused_variables)]
                                        Some(OwnedConstant::String(s)) => {
                                            trace!("    [{}]: Adding attribute \"{s}\" -> Default", class.this_class);
                                            attributes.insert(s.to_string(), String::from("\"default\""));
                                        }
                                        a => {
                                            panic!("Expected a string for \"add\", got: {a:?}");
                                        }
                                    }
                                },
                                "add" if &name_and_type.descriptor == "(Lnet/minecraft/registry/entry/RegistryEntry;D)Lnet/minecraft/entity/attribute/DefaultAttributeContainer$Builder;" => {
                                    match (constants.pop(), constants.pop()) {
                                        (Some(OwnedConstant::Double(d)), Some(OwnedConstant::String(s))) => {
                                            fn round(d: f64) -> f64 { (d * 10000.0).round() / 10000.0 }

                                            debug!("    [{}]: Adding attribute \"{s}\" -> {d}", class.this_class);
                                            attributes.insert(s.to_string(), format!("{}f64", round(d)));
                                        }
                                        (a, b) => {
                                            panic!("Expected a string and a double for \"add\", got: ({a:?}, {b:?})");
                                        }
                                    }
                                },
                                unk => panic!("Unknown attribute builder method: {unk} ({})", name_and_type.descriptor),
                            }

                        }
                    _ => {}
                }
            });
        }

        attributes
    }

    /// Find a class method,
    /// either in the class itself or in any of its superclasses.
    fn find_and_parse_class_method(
        class_name: &str,
        class_method: &str,
        jar: &CodeBundle,
    ) -> IndexMap<String, String> {
        if !class_name.starts_with("net/minecraft") {
            return IndexMap::new();
        } else if let Some(class) = jar.get(class_name) {
            if let Some(method) = class.methods.iter().find(|m| m.name == class_method) {
                return Self::parse_class_method_attributes(&class, method, jar);
            } else if let Some(super_class) = class.super_class.as_ref() {
                return Self::find_and_parse_class_method(super_class, class_method, jar);
            }
        }

        panic!("Could not find class method \"{class_method}\" in class \"{class_name}\"");
    }
}
