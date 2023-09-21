use bevy_ecs::change_detection::Res;
use bevy_ecs::prelude::Commands;
use log::info;
use valence::anvil::AnvilLevel;
use valence::prelude::{BiomeRegistry, DimensionTypeRegistry};
use valence::{ident, BlockState, ChunkPos, LayerBundle, Server};

pub fn setup(
    mut commands: Commands,
    server: Res<Server>,
    biomes: Res<BiomeRegistry>,
    dimensions: Res<DimensionTypeRegistry>,
) {
    let mut overworld_layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);

    let mut level = AnvilLevel::new(
        "/Users/tsatke/Library/Application Support/minecraft/saves/New World",
        &biomes,
    );

    for z in -8..8 {
        for x in -8..8 {
            let pos = ChunkPos::new(x, z);
            level.ignored_chunks.insert(pos);
            level.force_chunk_load(pos);
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

    commands.spawn((overworld_layer, level));

    info!("setup complete");
}
