use bevy_ecs::change_detection::Res;
use bevy_ecs::prelude::Commands;
use log::info;
use valence::prelude::{BiomeRegistry, DimensionTypeRegistry, UnloadedChunk};
use valence::{ident, BlockState, LayerBundle, Server};

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

    overworld_layer
        .chunk
        .set_block([start_x, 65, start_z + 2], BlockState::OAK_LOG);

    commands.spawn(overworld_layer);

    info!("setup complete");
}
