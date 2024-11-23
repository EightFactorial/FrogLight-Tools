use froglight_parse::file::protocol::{ProtocolType, ProtocolTypeArgs};

mod handle;

mod file;
pub use file::File;

mod result;
pub use result::Result;

mod state;
pub use state::State;
use state::Target;

impl super::PacketGenerator {
    /// Return the type of a [`ProtocolType`],
    /// generating the type if necessary.
    ///
    /// # Note
    /// This may pass back attributes for the type.
    #[must_use]
    pub fn generate_type(state: &State<Target>, proto: &ProtocolType, file: &mut File) -> Result {
        match proto {
            ProtocolType::Named(native) => match native.as_str() {
                "void" => Result::Void,
                other => Result::item_from_str(other),
            },
            ProtocolType::Inline(_, type_args) => Self::generate_args(state, type_args, file),
        }
    }

    /// Generate the type specified by the [`ProtocolTypeArgs`].
    ///
    /// # Note
    /// This may pass back attributes for the type.
    #[must_use]
    fn generate_args(state: &State<Target>, args: &ProtocolTypeArgs, file: &mut File) -> Result {
        match args {
            ProtocolTypeArgs::Array(args) => Self::handle_array(state, args, file),
            ProtocolTypeArgs::ArrayWithLengthOffset(args) => Self::handle_offset(state, args, file),
            ProtocolTypeArgs::Bitfield(args) => Self::handle_bitfield(state, args, file),
            ProtocolTypeArgs::Bitflags(args) => Self::handle_bitflags(state, args, file),
            ProtocolTypeArgs::Buffer(args) => Self::handle_buffer(state, args, file),
            ProtocolTypeArgs::Container(args) => Self::handle_container(state, args, file),
            ProtocolTypeArgs::EntityMetadata(args) => Self::handle_metadata(state, args, file),
            ProtocolTypeArgs::Mapper(args) => Self::handle_mapper(state, args, file),
            ProtocolTypeArgs::Option(option) => Self::handle_option(state, option, file),
            ProtocolTypeArgs::PString(args) => Self::handle_pstring(state, args, file),
            ProtocolTypeArgs::RegistryEntryHolder(args) => {
                Self::handle_registry_entry(state, args, file)
            }
            ProtocolTypeArgs::RegistryEntryHolderSet(args) => {
                Self::handle_registry_entry_set(state, args, file)
            }
            ProtocolTypeArgs::Switch(args) => Self::handle_switch(state, args, file),
            ProtocolTypeArgs::TopBitSetTerminatedArray(args) => {
                Self::handle_bitset_array(state, args, file)
            }
        }
    }
}
