use super::{Movement, PlayerActions};
use crate::mapas::{GameMap, Position};
use std::collections::VecDeque;

const PLAYER_STEP_DELAY: f32 = 0.15;

pub struct Warrior {
    position: Position,
    health: i32,
    score: i32,
    path: VecDeque<Position>,
    step_timer: f32,
}

impl Warrior {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            health: 100,
            score: 0,
            path: VecDeque::new(),
            step_timer: 0.0,
        }
    }
}

impl PlayerActions for Warrior {
    fn position(&self) -> Position {
        self.position
    }
    fn health(&self) -> i32 {
        self.health
    }
    fn score(&self) -> i32 {
        self.score
    }
    fn take_damage(&mut self, damage: i32) {
        self.health -= damage;
    }
    fn add_score(&mut self, points: i32) {
        self.score += points;
    }

    fn start_movement(&mut self, path: Vec<Position>) {
        self.path = path.into();
        self.step_timer = 0.0;
    }

    fn advance_movement(&mut self, dt: f32, map: &impl GameMap, occupied: &[Position]) -> Movement {
        self.step_timer -= dt;
        if self.step_timer > 0.0 {
            return Movement::Waiting;
        }
        let Some(next) = self.path.pop_front() else {
            return Movement::Blocked;
        };
        if !map.is_walkable(next) || occupied.contains(&next) {
            self.path.clear();
            return Movement::Blocked;
        }
        self.position = next;
        let finished = self.path.is_empty();
        if !finished {
            self.step_timer = PLAYER_STEP_DELAY;
        }
        Movement::Stepped {
            position: next,
            finished,
        }
    }

    fn remaining_path(&self) -> &VecDeque<Position> {
        &self.path
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/warrior.rs"]
mod tests;
