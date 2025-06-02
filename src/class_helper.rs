#![allow(dead_code)]

use cafebabe::{
    attributes::{AttributeData, AttributeInfo, BootstrapMethodEntry, CodeData},
    bytecode::Opcode,
    constant_pool::{BootstrapArgument, LiteralConstant, MethodHandle},
    ClassFile,
};
use froglight_dependency::dependency::minecraft::minecraft_code::CodeBundle;

pub(crate) trait ClassHelper {
    fn class_code(&self) -> &CodeData<'_> { self.class_method_code("<clinit>") }
    fn class_method_code(&self, method: &str) -> &CodeData<'_> {
        Self::class_optional_method_code(self, method)
            .expect("Class method does not contain a `Code` attribute!")
    }
    fn class_optional_method_code(&self, method: &str) -> Option<&CodeData<'_>>;

    fn class_bootstrap_entries(&self) -> &[BootstrapMethodEntry<'_>];
    fn class_bootstrap_methods(&self, index: u16) -> impl Iterator<Item = &MethodHandle<'_>> {
        self.class_bootstrap_entries()[index as usize].arguments.iter().filter_map(|arg| {
            if let BootstrapArgument::MethodHandle(handle) = arg {
                Some(handle)
            } else {
                None
            }
        })
    }
    fn class_bootstrap_code(
        &self,
        index: u16,
        classes: &CodeBundle,
        f: &mut impl FnMut(&Opcode<'_>),
    ) {
        for method in self.class_bootstrap_methods(index) {
            if let Some(class) = classes.get(&method.class_name) {
                let code = class.class_method_code(&method.member_ref.name);
                for (_, opcode) in &code.bytecode.as_ref().unwrap().opcodes {
                    f(opcode);
                    if let Opcode::Invokedynamic(invoke) = opcode {
                        self.class_bootstrap_code(invoke.attr_index, classes, f);
                    }
                }
            }
        }
    }

    fn iter_code_recursive(
        &self,
        initial: &[&Opcode<'_>],
        classes: &CodeBundle,
        mut f: impl FnMut(&Opcode<'_>),
    ) {
        for opcode in initial {
            f(opcode);
            if let Opcode::Invokedynamic(invoke) = opcode {
                self.class_bootstrap_code(invoke.attr_index, classes, &mut f);
            }
        }
    }
}

impl ClassHelper for ClassFile<'_> {
    fn class_optional_method_code(&self, method: &str) -> Option<&CodeData<'_>> {
        self.methods.iter().find_map(|m| {
            if m.name == method {
                for attr in &m.attributes {
                    if let AttributeInfo { data: AttributeData::Code(code), .. } = attr {
                        return Some(code);
                    }
                }
            }
            None
        })
    }

    fn class_bootstrap_entries(&self) -> &[BootstrapMethodEntry<'_>] {
        self.attributes
            .iter()
            .find_map(|attr| {
                if let AttributeInfo { data: AttributeData::BootstrapMethods(methods), .. } = attr {
                    Some(methods)
                } else {
                    None
                }
            })
            .expect("Class does not contain a `BootstrapMethods` attribute!")
    }
}

// -------------------------------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) enum OwnedConstant {
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    String(String),
    StringBytes(Vec<u8>),
}

impl From<&LiteralConstant<'_>> for OwnedConstant {
    fn from(constant: &LiteralConstant) -> Self {
        match constant {
            LiteralConstant::Integer(value) => OwnedConstant::Integer(*value),
            LiteralConstant::Float(value) => OwnedConstant::Float(*value),
            LiteralConstant::Long(value) => OwnedConstant::Long(*value),
            LiteralConstant::Double(value) => OwnedConstant::Double(*value),
            LiteralConstant::String(string) => OwnedConstant::String(string.clone().into_owned()),
            LiteralConstant::StringBytes(bytes) => OwnedConstant::StringBytes(bytes.to_vec()),
        }
    }
}
