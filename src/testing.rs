use test_script::{parse, Assert, Face, Gamemode, Interact, Line, Set};
use valence::interact_block::InteractBlockEvent;
use valence::inventory::HeldItem;
use valence::prelude::*;
use valence::testing::ScenarioSingleClient;

pub trait TestableEnvironment {
    fn new() -> Self;
    fn app(&mut self) -> &mut App;

    fn layer(&self) -> Entity;
    fn client(&self) -> Entity;
}

impl TestableEnvironment for ScenarioSingleClient {
    fn new() -> Self {
        ScenarioSingleClient::new()
    }

    fn app(&mut self) -> &mut App {
        &mut self.app
    }

    fn layer(&self) -> Entity {
        self.layer
    }

    fn client(&self) -> Entity {
        self.client
    }
}

pub fn eval_script<T>(input: &str)
where
    T: TestableEnvironment,
{
    let mut env = T::new();
    env.app().update();

    {
        // set up chunk layer
        let layer_entity = env.layer();
        let mut entity_mut = env.app().world.entity_mut(layer_entity);
        let mut layer = entity_mut.get_mut::<ChunkLayer>().unwrap();

        // insert unloadedchunks between -1..=1 and -1..=1
        // TODO: check the commands in the script and insert chunks as needed
        for x in -1..=1 {
            for z in -1..=1 {
                layer.insert_chunk([x, z], UnloadedChunk::new());
            }
        }
        layer.set_block(
            BlockPos::new(0, 0, 0),
            BlockState::from_kind(BlockKind::GrassBlock),
        );
    }

    env.app().update();

    parse(input).into_iter().for_each(|line| {
        match line {
            Line::Set(v) => eval_set(&mut env, v),
            Line::Assert(v) => eval_assert(&mut env, v),
            Line::Interact(v) => eval_interact(&mut env, v),
        };
        env.app().update();
    });
}

fn eval_set<E>(env: &mut E, set: Set)
where
    E: TestableEnvironment,
{
    match set {
        Set::Gamemode(game_mode) => {
            let mut q = env.app().world.query::<&mut GameMode>();
            let mut current_game_mode = q.get_single_mut(&mut env.app().world).unwrap();
            *current_game_mode = match game_mode {
                Gamemode::Survival => GameMode::Survival,
                Gamemode::Creative => GameMode::Creative,
                Gamemode::Adventure => GameMode::Adventure,
                Gamemode::Spectator => GameMode::Spectator,
            };
        }
        Set::Inventory(inv) => {
            let mut q = env.app().world.query::<&mut Inventory>();
            let mut current_inventory = q.get_single_mut(&mut env.app().world).unwrap();
            current_inventory.set_slot(
                inv.slot,
                ItemStack::new(
                    ItemKind::from_str(inv.item.as_str()).unwrap(),
                    inv.count,
                    None,
                ),
            );
        }
        Set::HeldItem(slot) => {
            let mut q = env.app().world.query::<&mut HeldItem>();
            let mut current_held_item = q.get_single_mut(&mut env.app().world).unwrap();
            current_held_item.set_slot(slot);
        }
    }
}

fn eval_assert<E>(env: &mut E, assert: Assert)
where
    E: TestableEnvironment,
{
    match assert {
        Assert::Position(pos, block) => {
            let mut q = env.app().world.query::<&ChunkLayer>();
            let layer = q.get_single(&env.app().world).unwrap();
            let actual_block_state = layer
                .block(BlockPos::new(pos.x, pos.y, pos.z))
                .map(|b| b.state)
                .or(Some(BlockState::AIR))
                .unwrap();
            let expected_block_state =
                BlockState::from_kind(BlockKind::from_str(block.id.as_str()).unwrap());
            assert_eq!(
                actual_block_state, expected_block_state,
                "block at {:?} is not {:?}, but {:?}",
                pos, expected_block_state, actual_block_state,
            );
        }
        Assert::Inventory(slot, stack) => {
            let mut q = env.app().world.query::<&Inventory>();
            let inventory = q.get_single(&env.app().world).unwrap();
            let actual_stack = inventory.slot(slot);
            if let Some(stack) = stack {
                let expected_stack =
                    ItemStack::new(ItemKind::from_str(stack.0.as_str()).unwrap(), stack.1, None);
                assert_eq!(
                    actual_stack, &expected_stack,
                    "inventory slot {} is not {:?}, but {:?}",
                    slot, expected_stack, actual_stack,
                );
            } else {
                assert!(actual_stack.is_empty());
            }
        }
    }
}

fn eval_interact<E>(env: &mut E, interact: Interact)
where
    E: TestableEnvironment,
{
    let position = BlockPos::new(
        interact.position.x,
        interact.position.y,
        interact.position.z,
    );
    let face = match interact.face {
        Face::Up => Direction::Up,
        Face::Down => Direction::Down,
        Face::North => Direction::North,
        Face::South => Direction::South,
        Face::East => Direction::East,
        Face::West => Direction::West,
    };
    let client = env.client();
    env.app().world.send_event(InteractBlockEvent {
        client,
        hand: Hand::Main,
        position,
        face,
        cursor_pos: Vec3::new(0.5, 0.5, 0.5), // FIXME: this may not do forever - compute this out of the direction/face
        head_inside_block: false,
        sequence: 0,
    });
}
