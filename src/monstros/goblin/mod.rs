use super::{MonsterAction, MonsterBehavior};
use crate::mapas::{GameMap, Position, dist};

pub struct Goblin {
    position: Position,
    health: i32,
}

impl Goblin {
    pub fn new(position: Position, health: i32) -> Self {
        Self { position, health }
    }
}

impl MonsterBehavior for Goblin {
    fn position(&self) -> Position {
        self.position
    }
    fn health(&self) -> i32 {
        self.health
    }
    fn take_damage(&mut self, damage: i32) {
        self.health -= damage;
    }

    fn act(
        &mut self,
        map: &impl GameMap,
        player: Position,
        occupied: &[Position],
    ) -> MonsterAction {
        if dist(self.position, player) == 1 {
            return MonsterAction::Attack(5);
        }
        let path = map.path(self.position, player, &[]);
        if let Some(&next) = path.first()
            && !occupied.contains(&next)
            && next != player
        {
            self.position = next;
            return MonsterAction::Move;
        }
        MonsterAction::Pass
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/goblin.rs"]
mod tests;
