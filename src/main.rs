use bevy_log::{Level, LogPlugin};
use justmine::{accept_connection, fell_out_of_world, place_block, remove_block, respawn, setup};
use valence::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "justmine=trace".to_string(),
            level: Level::INFO,
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                accept_connection,
                fell_out_of_world,
                remove_block,
                place_block,
                respawn,
                despawn_disconnected_clients,
            ),
        )
        .run();
}
