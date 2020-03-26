use crate::game::Game;
use fecs::{World, Write, Read, IntoQuery};
use crate::entity::{EntityProperties, EntityId};
use feather_core::network::packet::implementation::EntityProperties as PEntityProperties;
use feather_core::Position;
use crate::network::Network;

#[system]
pub fn poll_entity_properties_changed(game: &mut Game, world: &mut World) {
    let mut packets = Vec::new();

    for (mut properties, position, eid) in <(Write<EntityProperties>, Read<Position>, Read<EntityId>)>::query().iter_mut(world.inner_mut()) {
        if properties.dirty {
            debug!("Found dirty properties for entity with id {:?}", *eid);
            let packet = PEntityProperties {
                entity_id: eid.0,
                properties: properties.inner.clone()
            };
            properties.dirty = false;
            packets.push((packet, *position));
        }
    }

    for (packet, position) in packets {
        debug!("Trying to broadcast chunk update around {:?}", position);
        game.broadcast_chunk_update_boxed(&world, Box::new(packet), position.chunk(), None);
    }
}