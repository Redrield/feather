use crate::entity::{EntityId, EntityProperties};
use crate::game::Game;
use crate::TPS;
use bitflags::bitflags;
use feather_core::network::packet::implementation::{EntityEffect, RemoveEntityEffect, EntityStatus};
use feather_core::{StatusEffect, Position, PropertyModifier, ModifierOperation};
use fecs::{IntoQuery, Read, World, Write, RefMut, Ref};
use thiserror::Error;
use crate::network::Network;
use uuid::Uuid;

bitflags! {
    pub struct EffectFlags: i8 {
        const AMBIENT = 0x01;
        const SHOW_PARTICLES = 0x02;
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("The provided status effect {0:?} requires additional work by the server and cannot be stored as a BasicStatusEffect.")]
    InvalidStatusEffect(StatusEffect),
}

/// A status effect on an entity that doesn't require more work by the server than to broadcast effect packets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicStatusEffect {
    entity_id: EntityId,
    amplifier: u8,
    time_start: i64,
    duration: u64,
    effect_type: StatusEffect,
    flags: EffectFlags,
}

impl BasicStatusEffect {
    pub fn new(
        entity_id: EntityId,
        level: u8,
        duration: u64,
        effect_type: StatusEffect,
        flags: EffectFlags,
    ) -> Result<BasicStatusEffect, Error> {
        //TODO: Validate that the provided effect type is basic
        Ok(BasicStatusEffect {
            entity_id,
            amplifier: level - 1,
            time_start: -1,
            duration,
            effect_type,
            flags,
        })
    }

    pub fn create_packet(&self) -> EntityEffect {
        EntityEffect {
            entity_id: self.entity_id.0,
            effect_id: self.effect_type.protocol_id(),
            amplifier: self.amplifier as i8,
            duration: (self.duration / TPS) as i32,
            flags: self.flags.bits()
        }
    }
}

pub struct EntityBasicStatusEffects(pub Vec<BasicStatusEffect>);

pub struct SpeedEffect {
    entity_id: EntityId,
    amplifier: i8,
    start_time: u64,
    duration: u64,
    new: bool,
}

impl SpeedEffect {
    pub fn new(entity_id: EntityId, level: i8, duration: u64) -> SpeedEffect {
        SpeedEffect {
            entity_id,
            amplifier: level - 1,
            duration,
            start_time: 0,
            new: true
        }
    }
}

#[system]
pub fn update_speed_effects(game: &mut Game, world: &mut World) {
    let mut packets = Vec::new();

    let now = game.tick_count;

    for (mut effect, mut properties, position) in <(Write<SpeedEffect>, Write<EntityProperties>, Read<Position>)>::query().iter_mut(world.inner_mut()) {
        if effect.new {
            properties.get_property_mut("generic.movementSpeed").unwrap()
                .add_modifier(PropertyModifier::new(Uuid::parse_str("91AEAA56-376B-4498-935B-2F7F68070635").unwrap(), 0.2, ModifierOperation::Multiply));
            effect.start_time = now;
            effect.new = false;
            let packet = EntityEffect {
                entity_id: effect.entity_id.0,
                effect_id: StatusEffect::Speed.protocol_id(),
                amplifier: effect.amplifier,
                duration: effect.duration as i32,
                flags: EffectFlags::empty().bits()
            };
            packets.push((packet, *position));
        }
    }

    for (packet, position) in packets {
        game.broadcast_chunk_update_boxed(&world, Box::new(packet), position.chunk(), None);
    }
}

#[system]
pub fn update_basic_effects(game: &mut Game, world: &mut World) {
    let mut stale_effect_updates = Vec::new();
    let mut pending_effect_starts = Vec::new();

    let now = game.tick_count;

    // Look for any status effects that haven't been sent yet, and update them with the proper starting time.
    for (mut effects, position) in <(Write<EntityBasicStatusEffects>, Read<Position>)>::query().iter_mut(world.inner_mut()).filter(|(effects, _)| effects.0.iter().any(|effect| effect.time_start < 0))
    {
        for effect in effects.0.iter_mut() {
            if effect.time_start < 0 {
                effect.time_start = now as i64;
                pending_effect_starts.push((effect.create_packet(), *position))
            }
        }
    }

    for (packet, position) in pending_effect_starts.into_iter() {
        game.broadcast_chunk_update_boxed(&world, Box::new(packet), position.chunk(), None);
    }

    if game.tick_count % 5 == 0 {
        // Go through all entities, look for expired effects, keep track of pending packets + entity locations
        for (mut effects, position) in <(Write<EntityBasicStatusEffects>, Read<Position>)>::query().iter_mut(world.inner_mut()) {
            for (i, effect) in effects.0.clone().iter().enumerate() {
                let remaining_time = effect.time_start + (effect.duration as i64) - (now as i64);
                if remaining_time <= 0 {
                    debug!("{:?} is stale, scheduling for removal", effect);
                    let packet = RemoveEntityEffect {
                        entity_id: effect.entity_id.0,
                        effect_id: effect.effect_type.protocol_id()
                    };

                    stale_effect_updates.push((packet, *position));
                    effects.0.remove(i);
                }
            }
        }

        // Send packets to get clients to remove stale effects
        for (packet, position) in stale_effect_updates {
            debug!("Sending remove effect");
            game.broadcast_chunk_update_boxed(&world, Box::new(packet), position.chunk(), None);
        }
    }

    <(Read<EntityBasicStatusEffects>, Read<Position>, Read<Network>)>::query().par_for_each(world.inner(), |(effects, position, network)| {
        for effect in effects.0.iter() {
            let remaining_time = (effect.time_start as u64) + effect.duration - now;

            if remaining_time % 600 == 0 {
                let packet = EntityEffect {
                    entity_id: effect.entity_id.0,
                    effect_id: effect.effect_type.protocol_id(),
                    amplifier: effect.amplifier as i8,
                    duration: remaining_time as i32,
                    flags: effect.flags.bits(),
                };

                game.broadcast_chunk_update_boxed(&world, Box::new(packet), position.chunk(), None);
            }
        }
    });
}
