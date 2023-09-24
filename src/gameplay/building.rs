use bevy_ecs::prelude::*;
use valence::interact_block::InteractBlockEvent;
use valence::inventory::HeldItem;
use valence::prelude::*;

pub fn remove_block(
    clients: Query<&GameMode>,
    mut layers: Query<&mut ChunkLayer>,
    mut events: EventReader<DiggingEvent>,
) {
    let mut layer = layers.single_mut();

    events.iter().for_each(|event| {
        let Ok(game_mode) = clients.get(event.client) else {
            return;
        };

        if (*game_mode == GameMode::Creative && event.state == DiggingState::Start)
            || (*game_mode == GameMode::Survival && event.state == DiggingState::Stop)
        {
            layer.set_block(event.position, BlockState::AIR);
        }
    });
}

pub fn place_block(
    mut clients: Query<(&GameMode, &HeldItem, &Look, &mut Inventory)>,
    mut layers: Query<&mut ChunkLayer>,
    mut events: EventReader<InteractBlockEvent>,
) {
    let mut layer = layers.single_mut();

    events.iter().for_each(|event| {
        let (game_mode, held_item, look, mut inventory) = match clients.get_mut(event.client) {
            Ok(v) => v,
            Err(_) => return,
        };

        let slot = held_item.slot();
        let stack = inventory.slot(slot);
        if stack.is_empty() {
            // client is not holding anything, our work is done for this event
            return;
        }

        let target_position = event.position.get_in_direction(event.face);

        let block = match BlockKind::from_item_kind(stack.item) {
            Some(block) => block,
            None => return,
        };

        // don't decrement the stack amount in creative mode
        if game_mode == &GameMode::Survival {
            let count = stack.count;
            inventory.set_slot_amount(slot, count - 1);
        }

        let mut state = BlockState::from_kind(block);
        if block.props().contains(&PropName::Axis) {
            // respect placement orientation if the block has an axis property
            state = state.set(
                PropName::Axis,
                match event.face {
                    Direction::Down | Direction::Up => PropValue::Y,
                    Direction::North | Direction::South => PropValue::Z,
                    Direction::West | Direction::East => PropValue::X,
                },
            );
        }

        if block.props().contains(&PropName::Facing) {
            // we're placing a door, fence gate, or similar block
            let yaw = look.yaw % 360_f32;
            let yaw = if yaw > 180_f32 {
                yaw - 360_f32
            } else if yaw < -180_f32 {
                yaw + 360_f32
            } else {
                yaw
            };
            let facing = match look {
                _ if (-135_f32..-45_f32).contains(&yaw) => PropValue::East,
                _ if (-45_f32..45_f32).contains(&yaw) => PropValue::South,
                _ if (45_f32..135_f32).contains(&yaw) => PropValue::West,
                _ if (135_f32..=180_f32).contains(&yaw) | (-180_f32..-135_f32).contains(&yaw) => {
                    PropValue::North
                }
                _ => panic!("invalid look angle: {}", yaw),
            };
            state = state.set(PropName::Facing, facing);

            if block.props().contains(&PropName::Half) {
                // we're placing a door
                state = state.set(PropName::Half, PropValue::Lower);
                let (v, invert) = match facing {
                    PropValue::South => (event.cursor_pos.x, false),
                    PropValue::North => (event.cursor_pos.x, true),
                    PropValue::East => (event.cursor_pos.z, true),
                    PropValue::West => (event.cursor_pos.z, false),
                    _ => unreachable!(),
                };
                let hinge_right = if invert { v > 0.5 } else { v < 0.5 };
                let hinge = if hinge_right {
                    PropValue::Right
                } else {
                    PropValue::Left
                };

                if !hinge_right {
                    if let Some(block) =
                        layer.block(target_position.get_in_direction(match facing {
                            PropValue::South => Direction::East,
                            PropValue::North => Direction::West,
                            PropValue::East => Direction::North,
                            PropValue::West => Direction::South,
                            _ => unreachable!(),
                        }))
                    {
                        if is_door(block.state) {
                            state = state.set(PropName::Hinge, PropValue::Right);
                        }
                    }
                } else {
                    state = state.set(PropName::Hinge, hinge);
                }

                let upper = state.set(PropName::Half, PropValue::Upper);
                // FIXME:: check that that block is air, otherwise we can't place it
                layer.set_block(target_position.get_in_direction(Direction::Up), upper);
            }
        }
        layer.set_block(target_position, state);
    });
}

fn is_door(block: BlockState) -> bool {
    block.to_kind().to_str().contains("door")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{eval_script, TestableEnvironment};
    use std::ops::{Deref, DerefMut};
    use valence::testing::ScenarioSingleClient;

    const INVENTORY_SLOT: u16 = 36;

    struct BlockPlacementScenario {
        scenario: ScenarioSingleClient,
    }

    impl BlockPlacementScenario {
        fn new() -> Self {
            let mut scenario = ScenarioSingleClient::new();
            scenario.app.add_systems(Update, place_block);
            scenario.app.update();

            {
                // set up chunk layer
                let mut entity_mut = scenario.app.world.entity_mut(scenario.layer);
                let mut layer = entity_mut.get_mut::<ChunkLayer>().unwrap();
                // insert unloadedchunks between -1..=1 and -1..=1
                for x in -1..=1 {
                    for z in -1..=1 {
                        layer.insert_chunk([x, z], UnloadedChunk::new());
                    }
                }
                layer.set_block(
                    BlockPos::new(0, 0, 0),
                    BlockState::from_kind(BlockKind::GrassBlock),
                );
            }

            scenario.app.update();

            Self { scenario }
        }

        fn layer(&self) -> &ChunkLayer {
            let layer_entity = self.world.entity(self.scenario.layer);
            layer_entity.get::<ChunkLayer>().unwrap()
        }

        fn client_entity(&self) -> Entity {
            self.scenario.client
        }
    }

    impl Deref for BlockPlacementScenario {
        type Target = App;

        fn deref(&self) -> &Self::Target {
            &self.scenario.app
        }
    }

    impl DerefMut for BlockPlacementScenario {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.scenario.app
        }
    }

    #[test]
    fn test_assumptions() {
        /*
        Since valence is not stable yet, in this test, we test our assumptions, such as
        that the default game mode is survival.

        If this test fails, that means that something in valence changed, and we probably
        need to adapt our tests.
         */

        assert_eq!(GameMode::Survival, GameMode::default());
    }

    #[test]
    fn test_place_block_with_axis() {
        let mut scenario = BlockPlacementScenario::new();
        let client = scenario.client_entity();

        {
            // set up entity
            let mut q = scenario.world.query::<(&mut HeldItem, &mut Inventory)>();
            let (mut held_item, mut inventory) = q.get_single_mut(&mut scenario.world).unwrap();
            held_item.set_slot(INVENTORY_SLOT);
            inventory.set_slot(INVENTORY_SLOT, ItemStack::new(ItemKind::OakLog, 6, None));
        }

        for direction in [
            Direction::Up,
            Direction::Down,
            Direction::South,
            Direction::North,
            Direction::East,
            Direction::West,
        ] {
            scenario.world.send_event(InteractBlockEvent {
                client,
                hand: Hand::Main,
                position: BlockPos::new(0, 0, 0),
                face: direction,
                cursor_pos: match direction {
                    Direction::Down => Vec3::new(0.5, 0.0, 0.5),
                    Direction::Up => Vec3::new(0.5, 1.0, 0.5),
                    Direction::North => Vec3::new(0.5, 0.5, 0.0),
                    Direction::South => Vec3::new(0.5, 0.5, 1.0),
                    Direction::West => Vec3::new(0.0, 0.5, 0.5),
                    Direction::East => Vec3::new(1.0, 0.5, 0.5),
                },
                head_inside_block: false,
                sequence: 0,
            });
            scenario.update();
        }

        {
            let layer = scenario.layer();
            for direction in [
                Direction::Up,
                Direction::Down,
                Direction::South,
                Direction::North,
                Direction::East,
                Direction::West,
            ] {
                let target_block_pos = BlockPos::new(0, 0, 0).get_in_direction(direction);
                let block = layer
                    .block(target_block_pos)
                    .expect(format!("block at {:?} is None", target_block_pos).as_str());
                let expected_axis = match direction {
                    Direction::Down | Direction::Up => PropValue::Y,
                    Direction::North | Direction::South => PropValue::Z,
                    Direction::West | Direction::East => PropValue::X,
                };
                assert_eq!(
                    BlockState::from_kind(BlockKind::OakLog).set(PropName::Axis, expected_axis),
                    block.state,
                    "block in direction {:?}",
                    direction,
                );
            }

            let client_entity = scenario.world.entity(scenario.client_entity());
            let stack = client_entity
                .get::<Inventory>()
                .unwrap()
                .slot(INVENTORY_SLOT);
            assert!(stack.is_empty());
        };
    }

    struct PlaceBlockScenarioEnvironment<T: TestableEnvironment = ScenarioSingleClient> {
        env: T,
    }

    impl<T: TestableEnvironment> TestableEnvironment for PlaceBlockScenarioEnvironment<T> {
        fn new() -> Self {
            Self {
                env: {
                    let mut inner = T::new();
                    inner.app().add_systems(Update, place_block);
                    inner.app().update();
                    inner
                },
            }
        }

        fn app(&mut self) -> &mut App {
            self.env.app()
        }

        fn layer(&self) -> Entity {
            self.env.layer()
        }

        fn client(&self) -> Entity {
            self.env.client()
        }
    }

    #[test]
    fn test_place_block_simple() {
        eval_script::<PlaceBlockScenarioEnvironment>(
            r#"
            set gamemode survival
            set inventory slot 37 item oak_planks count 2
            set held_item 37
            
            assert position 0 0 0 block grass_block
            
            interact position 0 0 0 face up
            
            assert position 0 1 0 block oak_planks
            assert inventory slot 37 item oak_planks count 1
            assert inventory slot 36 empty
            "#,
        );
    }
}
