use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Added, Commands, Query, With};
use bevy_ecs::query::WorldQuery;
use log::info;
use valence::client::{Client, Username, VisibleChunkLayer, VisibleEntityLayers};
use valence::entity::{EntityLayerId, Position};
use valence::message::SendMessage;
use valence::player_list::PlayerListEntryBundle;
use valence::{ChunkLayer, EntityLayer, GameMode, UniqueId};

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct InitClientQuery {
    entity: Entity,
    uuid: &'static UniqueId,
    client: &'static mut Client,
    username: &'static Username,
    layer_id: &'static mut EntityLayerId,
    visible_chunk_layer: &'static mut VisibleChunkLayer,
    visible_entity_layers: &'static mut VisibleEntityLayers,
    pos: &'static mut Position,
    game_mode: &'static mut GameMode,
}

pub fn accept_connection(
    mut commands: Commands,
    mut clients: Query<InitClientQuery, Added<Client>>,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    for mut client in &mut clients {
        info!("new client connected");

        let layer = layers.single();

        client.layer_id.0 = layer;
        client.visible_chunk_layer.0 = layer;
        client.visible_entity_layers.0.insert(layer);
        client.pos.set([0.5, 65.5, 0.5]);
        *client.game_mode = GameMode::Creative;

        commands.spawn(PlayerListEntryBundle {
            uuid: *client.uuid,
            username: client.username.clone(),
            game_mode: *client.game_mode,
            ..Default::default()
        });

        client.client.send_chat_message("Welcome to the server!");
    }
}
