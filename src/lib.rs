use bevy_ecs::query::WorldQuery;
use log::info;
use valence::player_list::PlayerListEntryBundle;

use valence::prelude::*;
use valence::status::RequestRespawnEvent;

pub fn setup(
    mut commands: Commands,
    server: Res<Server>,
    biomes: Res<BiomeRegistry>,
    dimensions: Res<DimensionTypeRegistry>,
) {
    let mut overworld_layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);

    for z in -5..5 {
        for x in -5..5 {
            overworld_layer
                .chunk
                .insert_chunk([x, z], UnloadedChunk::new());
        }
    }

    let start_x = 0;
    let start_z = 0;
    for x in -5..5 {
        for z in -5..5 {
            overworld_layer
                .chunk
                .set_block([start_x + x, 64, start_z + z], BlockState::GRASS_BLOCK);
        }
    }

    commands.spawn(overworld_layer);

    info!("setup complete");
}

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

pub fn init_clients(
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
        *client.game_mode = GameMode::Survival;

        commands.spawn(PlayerListEntryBundle {
            uuid: *client.uuid,
            username: client.username.clone(),
            game_mode: *client.game_mode,
            ..Default::default()
        });

        client.client.send_chat_message("Welcome to the server!");
    }
}

pub fn digging(
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

/// A marker component that is added when a client dies.
/// This marker must be removed when the client respawns.
#[derive(Component)]
pub struct Dead;

pub fn fell_out_of_world(
    mut commands: Commands,
    mut clients: Query<(Entity, &mut Client, &Position), Without<Dead>>,
) {
    for (entity, mut client, pos) in &mut clients {
        if pos.y < 0.0 {
            commands.entity(entity).log_components();
            commands.entity(entity).insert(Dead);
            client.kill("What are you doing down there?");
        }
    }
}

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
    for event in events.iter() {
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
    }
}
