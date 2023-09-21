use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::{Commands, Query, With};
use valence::client::{VisibleChunkLayer, VisibleEntityLayers};
use valence::entity::EntityLayerId;
use valence::prelude::RespawnPosition;
use valence::status::RequestRespawnEvent;
use valence::{BlockPos, ChunkLayer, EntityLayer};

mod building;
mod environment;

pub use building::*;
pub use environment::*;

/// A marker component that is added when a client dies.
/// This marker must be removed when the client respawns.
#[derive(Component)]
pub struct Dead;

pub fn respawn(
    mut commands: Commands,
    mut clients: Query<
        (
            Entity,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut RespawnPosition,
        ),
        With<Dead>,
    >,
    mut events: EventReader<RequestRespawnEvent>,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    let layer = layers.single();
    events.iter().for_each(|event| {
        if let Ok((
            entity,
            mut layer_id,
            mut visible_chunk_layer,
            mut visible_entity_layers,
            mut respawn_pos,
        )) = clients.get_mut(event.client)
        {
            commands.entity(entity).remove::<Dead>();
            layer_id.0 = layer;
            visible_chunk_layer.0 = layer;
            visible_entity_layers.0.insert(layer);
            respawn_pos.pos = BlockPos::new(0, 65, 0);
        }
    });
}
