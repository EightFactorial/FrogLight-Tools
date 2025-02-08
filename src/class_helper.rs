#![allow(dead_code)]

use cafebabe::{
    attributes::{AttributeData, AttributeInfo, BootstrapMethodEntry, CodeData},
    bytecode::Opcode,
    constant_pool::{BootstrapArgument, MethodHandle},
    ClassFile,
};
use froglight_dependency::dependency::minecraft::minecraft_code::CodeBundle;

pub(crate) trait ClassHelper {
    fn class_code(&self) -> &CodeData<'_>;
    fn class_method_code(&self, method: &str) -> &CodeData<'_>;

    fn class_bootstrap(&self) -> &[BootstrapMethodEntry<'_>];
    fn class_bootstrap_methods(&self, index: u16) -> impl Iterator<Item = &MethodHandle<'_>> {
        self.class_bootstrap()[index as usize].arguments.iter().filter_map(|arg| {
            if let BootstrapArgument::MethodHandle(handle) = arg {
                Some(handle)
            } else {
                None
            }
        })
    }
    fn class_bootstrap_code(&self, index: u16, classes: &CodeBundle, f: &mut impl FnMut(&Opcode)) {
        for method in self.class_bootstrap_methods(index) {
            if let Some(class) = classes.get(&method.class_name) {
                let code = class.class_method_code(&method.member_ref.name);
                for (_, opcode) in &code.bytecode.as_ref().unwrap().opcodes {
                    match opcode {
                        Opcode::Invokedynamic(invoke) => {
                            class.class_bootstrap_code(invoke.attr_index, classes, f);
                        }
                        other => f(other),
                    }
                }
            }
        }
    }

    fn iter_code_recursive(
        &self,
        initial: &[&Opcode<'_>],
        classes: &CodeBundle,
        mut f: impl FnMut(&Opcode),
    ) {
        for opcode in initial {
            match opcode {
                Opcode::Invokedynamic(invoke) => {
                    self.class_bootstrap_code(invoke.attr_index, classes, &mut f);
                }
                other => f(other),
            }
        }
    }
}

impl ClassHelper for ClassFile<'_> {
    fn class_code(&self) -> &CodeData<'_> { self.class_method_code("<clinit>") }
    fn class_method_code(&self, method: &str) -> &CodeData<'_> {
        self.methods
            .iter()
            .find_map(|m| {
                if m.name == method {
                    for attr in &m.attributes {
                        if let AttributeInfo { data: AttributeData::Code(code), .. } = attr {
                            return Some(code);
                        }
                    }
                }
                None
            })
            .expect("Class method does not contain a `Code` attribute!")
    }

    fn class_bootstrap(&self) -> &[BootstrapMethodEntry<'_>] {
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
