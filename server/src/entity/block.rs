//! Components and utilities for block entities.
//!
//! # Registration
//! Block entities should be registered using `inventory::submit` in their
//! implementation file. For example:
//! `
//! inventory::submit!(BlockEntityRegistration::new(Block::Dirt, |lazy, entities| /* creation logic */));
//! `

use crate::blocks::BlockUpdateEvent;
use feather_blocks::Block;
use feather_core::world::ChunkMap;
use feather_core::BlockPosition;
use hashbrown::{HashMap, HashSet};
use shrev::{EventChannel, ReaderId};
use specs::world::{EntitiesRes, LazyBuilder};
use specs::{Builder, Component, DenseVecStorage, Entities, LazyUpdate, Read, System, World};
use std::ops::Deref;

/// Position of a block entity. The following conditions should generally
/// be upheld:
/// * Once created, a block entity's position should never change.
/// * This component should be used instead of `PositionComponent` for block
/// entities as it is not subject to floating-point errors.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockPositionComponent(pub BlockPosition);

impl Deref for BlockPositionComponent {
    type Target = BlockPosition;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Component for BlockPositionComponent {
    type Storage = DenseVecStorage<Self>;
}

pub trait BlockEntityCreator<'a>:
    Send + Sync + Fn(&'a LazyUpdate, &'a EntitiesRes) -> LazyBuilder<'a>
{
}

impl<'a, F> BlockEntityCreator<'a> for F where
    F: Send + Sync + Fn(&'a LazyUpdate, &'a EntitiesRes) -> LazyBuilder<'a>
{
}

/// Registration of a block entity. This is used to initialize
/// block entities when their corresponding blocks are created.
pub struct BlockEntityRegistration {
    /// Block which needs to be placed for the entity
    /// to be created.
    pub block: Block,
    /// Function which creates a new block entity, returning a `LazyBuilder` for continued component creation.
    pub creator: &'static dyn for<'a> BlockEntityCreator<'a>,
}

inventory::collect!(BlockEntityRegistration);

/// System to create block entities when blocks of the necessary
/// type are placed.
///
/// This system listens to `BlockUpdateEvent`.
#[derive(Default)]
pub struct BlockEntityCreateSystem {
    /// Internal mapping of `Block` -> `BlockEntityRegistration`.
    registry: HashMap<Block, &'static BlockEntityRegistration>,
    reader: Option<ReaderId<BlockUpdateEvent>>,
}

impl<'a> System<'a> for BlockEntityCreateSystem {
    type SystemData = (
        Read<'a, ChunkMap>,
        Read<'a, EventChannel<BlockUpdateEvent>>,
        Read<'a, LazyUpdate>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (chunk_map, block_events, lazy, entities) = data;

        // Prevent block entity from being added at the same position multiple times.
        let mut processed = HashSet::new();

        for event in block_events.read(self.reader.as_mut().unwrap()) {
            if !processed.insert(event.pos) {
                continue; // Position already handled
            }

            // Rather than using event.new_block, we check the current block in the
            // chunk map in case the block changed again since the event was triggered.
            let block = chunk_map.block_at(event.pos).unwrap_or(Block::Air);

            // Check for block entities which should be created, creating
            // if necessary.
            if let Some(registration) = self.registry.get(&block) {
                let create = registration.creator;
                create(&lazy, &entities)
                    .with(BlockPositionComponent(event.pos))
                    .build();
            }
        }
    }

    fn setup(&mut self, world: &mut World) {
        use specs::SystemData;
        Self::SystemData::setup(world);

        // Compile block entity registrations into a hash map.
        for registration in inventory::iter::<BlockEntityRegistration> {
            self.registry.insert(registration.block, registration);
        }

        self.reader = Some(world.fetch_mut::<EventChannel<_>>().register_reader());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::BlockUpdateCause;
    use crate::entity::EntitySpawnEvent;
    use crate::lazy::LazyUpdateExt;
    use crate::testframework as test;
    use specs::{NullStorage, WorldExt};

    inventory::submit!(BlockEntityRegistration {
        block: Block::Dirt,
        creator: &create_dirt_entity,
    });

    #[derive(Default)]
    struct DirtComponent;
    impl Component for DirtComponent {
        type Storage = NullStorage<Self>;
    }

    fn create_dirt_entity<'a>(lazy: &'a LazyUpdate, entities: &'a EntitiesRes) -> LazyBuilder<'a> {
        lazy.spawn_entity(&entities).with(DirtComponent)
    }

    #[test]
    fn spawn_block_entity() {
        let (mut world, mut dispatcher) = test::builder()
            .with(BlockEntityCreateSystem::default(), "")
            .build();

        test::populate_with_air(&mut world);

        world.register::<DirtComponent>();

        let pos = BlockPosition::new(0, 0, 0);
        world
            .fetch_mut::<ChunkMap>()
            .set_block_at(pos, Block::Dirt)
            .unwrap();

        let event = BlockUpdateEvent {
            cause: BlockUpdateCause::Test,
            pos,

            old_block: Block::Air,
            new_block: Block::Dirt,
        };
        test::trigger_event(&world, event);

        let mut reader = test::reader(&world);

        dispatcher.dispatch(&world);
        world.maintain();

        // Verify that entity was created.
        let events = test::triggered_events::<EntitySpawnEvent>(&world, &mut reader);

        assert_eq!(events.len(), 1);

        let first = events.first().unwrap();

        // Verify that correct components were added.
        let entity = first.entity;
        assert!(world.read_component::<DirtComponent>().contains(entity));
        let block_pos = world
            .read_component::<BlockPositionComponent>()
            .get(entity)
            .copied()
            .unwrap();
        assert_eq!(block_pos.0, pos);
    }
}