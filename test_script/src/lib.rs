pub fn parse<I>(input: I) -> Vec<Line>
where
    I: AsRef<str>,
{
    input
        .as_ref()
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(Line::parse)
        .collect()
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Line {
    Set(Set),
    Assert(Assert),
    Interact(Interact),
}

impl Line {
    pub fn parse(input: &str) -> Self {
        let mut fragments = input.split(char::is_whitespace);
        let cmd = fragments.next();
        match cmd {
            Some("set") => Self::Set(Set::parse(fragments)),
            Some("assert") => Self::Assert(Assert::parse(fragments)),
            Some("interact") => Self::Interact(Interact::parse(fragments)),
            _ => panic!("unknown command: {:?}", cmd),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Assert {
    Position(Position, Block),
    Inventory(u16, Option<(String, i8)>),
}

impl Assert {
    pub fn parse<'a, I>(input: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut fragments = input.into_iter();
        let cmd = fragments.next();
        match cmd {
            Some("position") => Self::Position(
                Position::parse(fragments.by_ref()),
                Block::parse(fragments.by_ref()),
            ),
            Some("inventory") => {
                fragments
                    .next()
                    .filter(|&s| s == "slot")
                    .expect("expected 'slot' keyword");
                let slot = fragments.next().unwrap().parse().unwrap();

                let next = fragments.next().unwrap();
                let item = if next != "empty" {
                    Some(next)
                        .filter(|&s| s == "item")
                        .expect("expected 'item' keyword");
                    let item = fragments.next().unwrap().to_string();
                    fragments
                        .next()
                        .filter(|&s| s == "count")
                        .expect("expected 'count' keyword");
                    let count = fragments.next().unwrap().parse().unwrap();
                    Some((item, count))
                } else {
                    None
                };

                Self::Inventory(slot, item)
            }
            _ => panic!("unknown command: {:?}", cmd),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Interact {
    pub position: Position,
    pub face: Face,
}

impl Interact {
    pub fn parse<'a, I>(input: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut fragments = input.into_iter();
        fragments
            .next()
            .filter(|&s| s == "position")
            .expect("expected 'position' keyword");
        let position = Position::parse(fragments.by_ref());
        fragments
            .next()
            .filter(|&s| s == "face")
            .expect("expected 'face' keyword");
        let face = Face::parse(fragments.by_ref());
        Self { position, face }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Face {
    Up,
    Down,
    North,
    South,
    East,
    West,
}

impl Face {
    pub fn parse<'a, I>(input: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut fragments = input.into_iter();
        let face = fragments.next();
        match face {
            Some("up") => Self::Up,
            Some("down") => Self::Down,
            Some("north") => Self::North,
            Some("south") => Self::South,
            Some("east") => Self::East,
            Some("west") => Self::West,
            _ => panic!("unknown face: {:?}", face),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Position {
    pub fn parse<'a, I>(input: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut fragments = input.into_iter();
        let x = fragments.next().unwrap().parse().unwrap();
        let y = fragments.next().unwrap().parse().unwrap();
        let z = fragments.next().unwrap().parse().unwrap();
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Block {
    pub id: String,
}

impl Block {
    pub fn parse<'a, I>(input: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut fragments = input.into_iter();
        fragments
            .next()
            .filter(|&s| s == "block")
            .expect("expected 'block' keyword");
        let id = fragments.next().unwrap().to_string();
        Self { id }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Set {
    Gamemode(Gamemode),
    Inventory(Inventory),
    HeldItem(u16),
}

impl Set {
    pub fn parse<'a, I>(input: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut fragments = input.into_iter();
        let cmd = fragments.next();
        match cmd {
            Some("gamemode") => Self::Gamemode(Gamemode::parse(fragments)),
            Some("inventory") => Self::Inventory(Inventory::parse(fragments)),
            Some("held_item") => Self::HeldItem(fragments.next().unwrap().parse().unwrap()),
            _ => panic!("unknown command: {:?}", cmd),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Gamemode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl Gamemode {
    pub fn parse<'a, I>(input: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut fragments = input.into_iter();
        let mode = fragments.next();
        match mode {
            Some("survival") => Self::Survival,
            Some("creative") => Self::Creative,
            Some("adventure") => Self::Adventure,
            Some("spectator") => Self::Spectator,
            _ => panic!("unknown gamemode: {:?}", mode),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Inventory {
    pub slot: u16,
    pub item: String,
    pub count: i8,
}

impl Inventory {
    pub fn parse<'a, I>(input: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut fragments = input.into_iter();
        fragments
            .next()
            .filter(|&s| s == "slot")
            .expect("expected 'slot' keyword");
        let slot = fragments.next().unwrap().parse().unwrap();

        fragments
            .next()
            .filter(|&s| s == "item")
            .expect("expected 'item' keyword");
        let item = fragments.next().unwrap().to_string();

        fragments
            .next()
            .filter(|&s| s == "count")
            .expect("expected 'count' keyword");
        let count = fragments.next().unwrap().parse().unwrap();

        Self { slot, item, count }
    }
}
