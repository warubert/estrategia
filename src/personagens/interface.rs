use crate::mapas::{GameMap, Position};
use std::collections::VecDeque;

pub trait PlayerActions {
    fn position(&self) -> Position;
    fn health(&self) -> i32;
    fn score(&self) -> i32;
    fn take_damage(&mut self, damage: i32);
    fn add_score(&mut self, points: i32);
    fn start_movement(&mut self, path: Vec<Position>);
    fn advance_movement(&mut self, dt: f32, map: &impl GameMap, occupied: &[Position]) -> Movement;
    fn remaining_path(&self) -> &VecDeque<Position>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum Movement {
    Waiting,
    Stepped { position: Position, finished: bool },
    Blocked,
}
