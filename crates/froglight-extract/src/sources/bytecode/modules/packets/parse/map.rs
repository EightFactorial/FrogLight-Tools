use cafebabe::{bytecode::ByteCode, constant_pool::MemberRef, ClassFile};

use super::CodecType;
use crate::bundle::ExtractBundle;

#[allow(clippy::unnecessary_wraps)]
pub(super) fn create_map<'a>(
    _classfile: &'a ClassFile<'_>,
    _code: &'a ByteCode<'_>,
    _index: usize,
) -> anyhow::Result<CodecType<'a>> {
    Ok(CodecType::Unit)
}

#[allow(clippy::unnecessary_wraps)]
pub(super) fn parse_map(
    _classfile: &ClassFile<'_>,
    _key: &MemberRef<'_>,
    _value: &MemberRef<'_>,
    _data: &ExtractBundle<'_>,
) -> anyhow::Result<Vec<String>> {
    Ok(Vec::new())
}
