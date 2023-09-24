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
    use valence::testing::ScenarioSingleClient;

    #[test]
    fn test_place_block_simple() {
        const INVENTORY_SLOT: u16 = 36;

        let mut scenario = ScenarioSingleClient::new();
        scenario.app.add_systems(Update, place_block);
        scenario.app.update();

        {
            // set up entity
            let mut entity_mut = scenario.app.world.entity_mut(scenario.client);
            *entity_mut.get_mut::<GameMode>().unwrap() = GameMode::Survival;
            entity_mut.get_mut::<HeldItem>().unwrap().set_slot(36);
            entity_mut
                .get_mut::<Inventory>()
                .unwrap()
                .set_slot(INVENTORY_SLOT, ItemStack::new(ItemKind::OakPlanks, 2, None));
        }

        {
            // set up chunk layer
            let mut entity_mut = scenario.app.world.entity_mut(scenario.layer);
            let mut layer = entity_mut.get_mut::<ChunkLayer>().unwrap();
            layer.insert_chunk([0, 0], UnloadedChunk::new());
            layer.set_block(
                BlockPos::new(0, 0, 0),
                BlockState::from_kind(BlockKind::GrassBlock),
            );
        }

        // place a block
        scenario.app.world.send_event(InteractBlockEvent {
            client: scenario.client,
            hand: Hand::Main,
            position: BlockPos::new(0, 0, 0),
            face: Direction::Up,
            cursor_pos: Vec3::new(0.5, 1.0, 0.5),
            head_inside_block: false,
            sequence: 0,
        });

        scenario.app.update();

        {
            let layer_entity = scenario.app.world.entity(scenario.layer);
            let layer = layer_entity.get::<ChunkLayer>().unwrap();
            let block = layer.block(BlockPos::new(0, 1, 0));
            assert_eq!(
                BlockState::from_kind(BlockKind::OakPlanks),
                block.unwrap().state,
            );

            let client_entity = scenario.app.world.entity(scenario.client);
            let stack = client_entity
                .get::<Inventory>()
                .unwrap()
                .slot(INVENTORY_SLOT);
            assert_eq!(stack.item, ItemKind::OakPlanks);
            assert_eq!(stack.count, 1);
            assert_eq!(stack.nbt, None);
        };
    }

    #[test]
    fn test_place_block_with_axis() {}
}
