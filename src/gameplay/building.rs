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

    for event in events.iter() {
        let Ok(game_mode) = clients.get(event.client) else {
            continue;
        };

        if (*game_mode == GameMode::Creative && event.state == DiggingState::Start)
            || (*game_mode == GameMode::Survival && event.state == DiggingState::Stop)
        {
            layer.set_block(event.position, BlockState::AIR);
        }
    }
}

pub fn place_block(
    mut clients: Query<(&GameMode, &HeldItem, &Look, &mut Inventory)>,
    mut layers: Query<&mut ChunkLayer>,
    mut events: EventReader<InteractBlockEvent>,
) {
    let mut layer = layers.single_mut();

    for event in events.iter() {
        let (game_mode, held_item, look, mut inventory) = match clients.get_mut(event.client) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let slot = held_item.slot();
        let stack = inventory.slot(slot);
        if stack.is_empty() {
            // client is not holding anything, our work is done for this event
            continue;
        }

        let target_position = event.position.get_in_direction(event.face);

        let block = match BlockKind::from_item_kind(stack.item) {
            Some(block) => block,
            None => continue,
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
                // FIXME: if hinge left, but left of the door is another door, the hinge should be on the right
                state = state.set(PropName::Hinge, hinge);

                let upper = state.clone().set(PropName::Half, PropValue::Upper);
                layer.set_block(target_position.get_in_direction(Direction::Up), upper);
            }
        }
        layer.set_block(target_position, state);
    }
}
