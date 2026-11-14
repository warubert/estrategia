pub type Position = (usize, usize);

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    Wall,
    Floor,
}

pub trait GameMap {
    fn tile(&self, position: Position) -> Option<Tile>;
    fn path(&self, start: Position, goal: Position, blocked: &[Position]) -> Vec<Position>;

    fn is_walkable(&self, position: Position) -> bool {
        self.tile(position) == Some(Tile::Floor)
    }
}
