#![allow(clippy::ptr_arg, dead_code)]

use std::{borrow::Cow, str::CharIndices};

use cafebabe::descriptor::{BaseType, FieldType, MethodDescriptor, ReturnDescriptor, Ty};

pub(crate) trait MethodDescriptorTrait<'a>: Sized {
    fn parse(chars: &Cow<'a, str>) -> Option<Self>;
}
impl<'a> MethodDescriptorTrait<'a> for MethodDescriptor<'a> {
    fn parse(chars: &Cow<'a, str>) -> Option<Self> {
        let mut chars_idx = chars.char_indices();

        // Method descriptors must start with a '('
        let (_, char) = chars_idx.next()?;
        if char != '(' {
            return None;
        }

        let mut parameters: Vec<FieldType> = Vec::new();
        'done: loop {
            // preserve the next item for use in the FieldType parser
            parameters.push(match chars_idx.as_str().chars().next()? {
                ')' => {
                    chars_idx.next(); // consume the final ')'
                    break 'done;
                }
                _ => FieldType::parse_from_chars_idx(chars, &mut chars_idx)?,
            });
        }

        let result = ReturnDescriptor::parse(chars, &mut chars_idx)?;
        Some(MethodDescriptor { parameters, result })
    }
}

pub(crate) trait ReturnDescriptorTrait<'a>: Sized {
    fn parse(chars: &Cow<'a, str>, chars_idx: &mut CharIndices) -> Option<Self>;
}
impl<'a> ReturnDescriptorTrait<'a> for ReturnDescriptor<'a> {
    fn parse(chars: &Cow<'a, str>, chars_idx: &mut CharIndices) -> Option<Self> {
        // preserve the next item for use in the FieldType parser
        Some(match chars_idx.as_str().chars().next()? {
            'V' => {
                chars_idx.next(); // for correctness
                ReturnDescriptor::Void
            }
            _ => ReturnDescriptor::Return(FieldType::parse_from_chars_idx(chars, chars_idx)?),
        })
    }
}

pub(crate) trait FieldTypeTrait<'a>: Sized {
    fn parse(chars: &Cow<'a, str>) -> Option<Self> {
        let mut chars_idx = chars.char_indices();
        Self::parse_from_chars_idx(chars, &mut chars_idx)
    }

    fn parse_from_chars_idx(chars: &Cow<'a, str>, chars_idx: &mut CharIndices) -> Option<Self>;
}
impl<'a> FieldTypeTrait<'a> for FieldType<'a> {
    fn parse_from_chars_idx(chars: &Cow<'a, str>, chars_idx: &mut CharIndices) -> Option<Self> {
        let mut field = None::<Ty>;
        let mut array_depth = 0;

        while let Some(ch) = chars_idx.next().map(|(_, ch)| ch) {
            match ch {
                'L' => {
                    field = Some(Ty::Object(parse_object(chars, chars_idx)?));
                    break;
                }
                '[' => {
                    array_depth += 1;

                    // A field descriptor representing an array type is valid only if it represents
                    // a type with 255 or fewer dimensions.  see: https://docs.oracle.com/javase/specs/jvms/se18/html/jvms-4.html#jvms-4.3.2
                    if array_depth > 255 {
                        return None;
                    }
                }
                ch => {
                    field = Some(Ty::Base(BaseType::parse(ch)?));
                    break;
                }
            };
        }

        let field = field?;
        if array_depth > 0 {
            Some(FieldType::Array { dimensions: array_depth, ty: field })
        } else {
            Some(FieldType::Ty(field))
        }
    }
}

/// Parses the object less the beginning L, e.g. this expects
/// `java/lang/Object;`
fn parse_object<'a>(chars: &Cow<'a, str>, chars_idx: &mut CharIndices) -> Option<Cow<'a, str>> {
    let start_idx = chars_idx.next().map(|ch_idx| ch_idx.0)?;
    let end_idx = chars_idx.find_map(|(idx, ch)| if ch == ';' { Some(idx) } else { None })?;

    // Because a Cow can be either Borrowed or Owned, we need to create an Owned
    // String in the case that it's not a reference. This should be rare, if
    // ever.
    let object = match *chars {
        Cow::Borrowed(chars) => Cow::Borrowed(chars.get(start_idx..end_idx)?),
        Cow::Owned(ref chars) => Cow::Owned(chars.get(start_idx..end_idx)?.to_string()),
    };
    Some(object)
}

pub(crate) trait BaseTypeTrait {
    fn parse(ch: char) -> Option<BaseType>;
}
impl BaseTypeTrait for BaseType {
    fn parse(ch: char) -> Option<BaseType> {
        match ch {
            'B' => Some(Self::Byte),
            'C' => Some(Self::Char),
            'D' => Some(Self::Double),
            'F' => Some(Self::Float),
            'I' => Some(Self::Int),
            'J' => Some(Self::Long),
            'S' => Some(Self::Short),
            'Z' => Some(Self::Boolean),
            _ => None,
        }
    }
}
