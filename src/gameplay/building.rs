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
    mut clients: Query<(&GameMode, &HeldItem, &mut Inventory)>,
    mut layers: Query<&mut ChunkLayer>,
    mut events: EventReader<InteractBlockEvent>,
) {
    let mut layer = layers.single_mut();

    for event in events.iter() {
        if let Ok((game_mode, held_item, mut inventory)) = clients.get_mut(event.client) {
            let slot = held_item.slot();
            let stack = inventory.slot(slot);
            if stack.is_empty() {
                // client is not holding anything, our work is done for this event
                continue;
            }

            let target_position = event.position.get_in_direction(event.face);

            if let Some(block) = BlockKind::from_item_kind(stack.item) {
                // don't decrement the stack amount in creative mode
                if game_mode == &GameMode::Survival {
                    let count = stack.count;
                    inventory.set_slot_amount(slot, count - 1);
                }

                let state = BlockState::from_kind(block).set(
                    PropName::Axis,
                    match event.face {
                        Direction::Down | Direction::Up => PropValue::Y,
                        Direction::North | Direction::South => PropValue::Z,
                        Direction::West | Direction::East => PropValue::X,
                    },
                );
                layer.set_block(target_position, state);
            }
        }
    }
}
