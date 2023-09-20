use bevy_log::{Level, LogPlugin};
use justmine::{digging, fell_out_of_world, init_clients, respawn, setup};
use valence::app::{App, PluginGroup, Startup, Update};
use valence::client::despawn_disconnected_clients;
use valence::DefaultPlugins;

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
                init_clients,
                fell_out_of_world,
                digging,
                respawn,
                despawn_disconnected_clients,
            ),
        )
        .run();
}
