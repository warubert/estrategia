use crate::mapas::{GameMap, Position};

#[derive(Debug, PartialEq, Eq)]
pub enum MonsterAction {
    Attack(i32),
    Move,
    Pass,
}

pub trait MonsterBehavior {
    fn position(&self) -> Position;
    fn health(&self) -> i32;
    fn take_damage(&mut self, damage: i32);
    fn act(&mut self, map: &impl GameMap, player: Position, occupied: &[Position])
    -> MonsterAction;
}
