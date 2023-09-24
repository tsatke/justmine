use crate::Dead;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Commands, Query, Without};
use valence::client::Client;
use valence::entity::Position;

pub fn fell_out_of_world(
    mut commands: Commands,
    mut clients: Query<(Entity, &mut Client, &Position), Without<Dead>>,
) {
    clients.for_each_mut(|(entity, mut client, pos)| {
        if pos.y < -64.0 {
            commands.entity(entity).insert(Dead);
            client.kill("What are you doing down there?");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use valence::prelude::*;
    use valence::testing::create_mock_client;

    #[test]
    fn test_fell_out_of_world() {
        let mut app = App::new();
        app.add_systems(Update, fell_out_of_world);

        let (client, _) = create_mock_client("test");
        let entity = app.world.spawn(client).id();

        app.update();

        let mut entity_mut = app.world.get_entity_mut(entity).unwrap();
        assert!(entity_mut.get::<Dead>().is_none());
        entity_mut
            .get_mut::<Position>()
            .expect("position component is missing")
            .y = -64.1;

        app.update();

        let entity_ref = app.world.get_entity(entity).unwrap();
        entity_ref
            .get::<Dead>()
            .expect("dead component is missing, but should have been inserted by system");
    }
}
