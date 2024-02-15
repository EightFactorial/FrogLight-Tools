//! Ok, this gets a *bit* complicated, but I'll try to explain it as best as I
//! can here.
//!
//! Here, we have our list of states and packets. We want to get the fields for
//! each packet. That's not too hard, right? We just need to find the class for
//! the packet and then get the fields from that class.
//!
//! But here's the thing: the field of the packet are not one-to-one with the
//! data sent over the network. For example, a packet might have multiple ints
//! that are used as masks, but the actual data sent over the network is a
//! single byte.
//!
//! The solution used here is to get the constructor of the packet class that
//! takes a `PacketByteBuf`. From there, we can read every function call that
//! the packet makes to the `PacketByteBuf` and use that to determine the fields
//! of the packet.

use std::{borrow::Cow, collections::BTreeMap};

use tracing::error;

use self::output::Output;
use super::packets::ProtocolState;
use crate::classmap::ClassMap;

mod find;

mod resolve;

mod output;

pub(super) fn get_fields<'a>(
    states: &'a BTreeMap<Cow<'_, str>, ProtocolState<'_>>,
    classmap: &'a ClassMap,
) -> serde_json::Value {
    let mut value = serde_json::Value::default();

    for packets in states.values() {
        for packet in packets.clientbound.iter().chain(packets.serverbound.iter()) {
            // Skip the `BundleS2CPacket` as it's a special case
            if packet == "net/minecraft/network/packet/s2c/play/BundleS2CPacket" {
                continue;
            }

            // Find the class for the packet
            let Some(class) = classmap.get(packet) else {
                unreachable!("Could not find class for packet {packet}");
            };

            match (find::bytebuf_reader(&class), find::bytebuf_writer(&class)) {
                // If we have both a reader and a writer, we can compare them
                (Some(reader), Some(writer)) => {
                    let result = resolve::compare_methods(&class, reader, writer, classmap);

                    let packet: &str = packet;
                    value[packet] = serde_json::to_value(result).unwrap();
                }
                // If we only have a reader, resolve it
                (Some(reader), None) => {
                    let result = resolve::resolve_reader(&class, reader);
                    let result = Output::Unnamed(result.into_iter().map(|(_, ty)| ty).collect());

                    let packet: &str = packet;
                    value[packet] = serde_json::to_value(result).unwrap();
                }
                // If we only have a writer, resolve it
                (None, Some(writer)) => {
                    let result = resolve::resolve_writer(&class, writer);
                    let result = Output::Unnamed(result.into_iter().map(|(_, ty)| ty).collect());

                    let packet: &str = packet;
                    value[packet] = serde_json::to_value(result).unwrap();
                }
                // If we have neither, log an error
                (None, None) => {
                    error!("Could not find reader or writer for packet {packet}");
                }
            }
        }
    }

    value
}
