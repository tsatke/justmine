use crate::Dead;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, Query, Without};
use valence::client::Client;
use valence::entity::Position;

pub fn fell_out_of_world(
    mut commands: Commands,
    mut clients: Query<(Entity, &mut Client, &Position), Without<Dead>>,
) {
    for (entity, mut client, pos) in &mut clients {
        if pos.y < 0.0 {
            commands.entity(entity).insert(Dead);
            client.kill("What are you doing down there?");
        }
    }
}
