use crate::entities::DmgText;
#[cfg(test)]
use crate::mapas::Tile;
use crate::mapas::{GameMap, MAP, Mapa1, Position, to_screen, to_tile};
use crate::monstros::{Goblin, MonsterAction, MonsterBehavior};
use crate::personagens::{Movement, PlayerActions, Warrior};

const MONSTER_TURN_DELAY: f32 = 0.3;
const DEFAULT_CAMERA: (f32, f32) = (640.0, 50.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Turn {
    Player,
    PlayerMoving,
    Monster(usize),
}

pub struct Game<M: GameMap = Mapa1, P: PlayerActions = Warrior, N: MonsterBehavior = Goblin> {
    map: M,
    cam: (f32, f32),
    player: P,
    turn: Turn,
    monster_turn_timer: f32,
    monsters: Vec<N>,
    texts: Vec<DmgText>,
    next_text_id: u64,
    gold: Vec<(usize, usize)>,
}

impl Game {
    pub fn new() -> Self {
        Self::with_camera(DEFAULT_CAMERA)
    }

    fn with_camera(cam: (f32, f32)) -> Self {
        Game {
            map: Mapa1::new(),
            cam,
            player: Warrior::new((2, 2)),
            turn: Turn::Player,
            monster_turn_timer: 0.0,
            monsters: vec![
                Goblin::new((8, 8), 30),
                Goblin::new((12, 4), 30),
                Goblin::new((15, 12), 30),
            ],
            texts: vec![],
            next_text_id: 0,
            gold: vec![(3, 3), (10, 2), (16, 5), (6, 14), (17, 17)],
        }
    }
}

impl<M: GameMap, P: PlayerActions, N: MonsterBehavior> Game<M, P, N> {
    pub fn player(&self) -> &P {
        &self.player
    }

    pub fn map(&self) -> &M {
        &self.map
    }

    pub fn monsters(&self) -> &[N] {
        &self.monsters
    }

    pub fn gold(&self) -> &[(usize, usize)] {
        &self.gold
    }

    pub fn floating_texts(&self) -> &[DmgText] {
        &self.texts
    }

    pub fn camera(&self) -> (f32, f32) {
        self.cam
    }

    pub fn is_over(&self) -> bool {
        self.player.health() <= 0 || self.monsters.is_empty()
    }

    pub fn is_player_action_turn(&self) -> bool {
        self.turn == Turn::Player && !self.is_over()
    }

    pub fn is_player_moving(&self) -> bool {
        self.turn == Turn::PlayerMoving
    }

    pub fn can_attack(&self) -> bool {
        let player = self.player.position();
        self.monsters
            .iter()
            .any(|monster| crate::mapas::dist(player, monster.position()) == 1)
    }

    pub fn end_player_turn(&mut self) {
        if self.is_player_action_turn() {
            self.start_monster_turns();
        }
    }

    pub fn hovered_tile(&self, mouse: (f32, f32)) -> Option<Position> {
        let tile = to_tile(mouse.0, mouse.1, self.cam);
        (tile.0 >= 0 && tile.1 >= 0 && tile.0 < MAP as isize && tile.1 < MAP as isize)
            .then_some((tile.0 as usize, tile.1 as usize))
    }

    pub fn preview_path(&self, mouse: (f32, f32)) -> Vec<Position> {
        if self.turn == Turn::PlayerMoving {
            return self.player.remaining_path().iter().copied().collect();
        }

        if self.turn == Turn::Player
            && !self.is_over()
            && let Some(tile) = self.hovered_tile(mouse)
        {
            return self.player_path(tile);
        }

        vec![]
    }

    pub fn turn_text(&self) -> String {
        match self.turn {
            Turn::Player => "Turno: jogador".to_owned(),
            Turn::PlayerMoving => "Turno: jogador (movendo)".to_owned(),
            Turn::Monster(i) => format!("Turno: monstro {}/{}", i + 1, self.monsters.len()),
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        //victory/loss logic
        if self.is_over() {
            return true;
        }

        //update text animations
        self.texts.retain_mut(|t| {
            t.life -= dt;
            t.y -= 20. * dt;
            t.life > 0.
        });

        match self.turn {
            Turn::Player => {}
            Turn::PlayerMoving => self.advance_player_movement(dt),
            Turn::Monster(_) => self.advance_monster_turn(dt),
        }

        self.is_over()
    }

    fn player_path(&self, target: (usize, usize)) -> Vec<(usize, usize)> {
        if !self.map.is_walkable(target) {
            return vec![];
        }
        let blocked: Vec<Position> = self
            .monsters
            .iter()
            .map(|m| m.position())
            .filter(|&position| position != target)
            .collect();
        let mut path = self.map.path(self.player.position(), target, &blocked);
        if path.len() > 1 && self.monsters.iter().any(|m| m.position() == target) {
            path.pop();
        }
        path
    }

    pub fn move_player_to(&mut self, target: Position) -> bool {
        if self.turn != Turn::Player || self.is_over() {
            return false;
        }
        let path = self.player_path(target);
        if path.is_empty() {
            return false;
        }
        self.player.start_movement(path);
        self.turn = Turn::PlayerMoving;
        self.advance_player_movement(0.0);
        true
    }

    pub fn attack_monster_at(&mut self, target: Position) -> bool {
        if self.turn != Turn::Player || self.is_over() {
            return false;
        }
        let player = self.player.position();
        let Some(i) = self.monsters.iter().position(|monster| {
            monster.position() == target && crate::mapas::dist(player, target) == 1
        }) else {
            return false;
        };
        self.damage_monster(i, 10);
        if !self.is_over() {
            self.start_monster_turns();
        }
        true
    }

    fn advance_player_movement(&mut self, dt: f32) {
        if self.turn != Turn::PlayerMoving || self.is_over() {
            return;
        }
        let occupied: Vec<_> = self.monsters.iter().map(|m| m.position()).collect();
        match self.player.advance_movement(dt, &self.map, &occupied) {
            Movement::Waiting => {}
            Movement::Blocked => self.start_monster_turns(),
            Movement::Stepped {
                position: (nx, ny),
                finished,
            } => {
                if let Some(i) = self.gold.iter().position(|&g| g == (nx, ny)) {
                    self.gold.remove(i);
                    self.player.add_score(100);
                    let (sx, sy) = to_screen(nx, ny, self.cam);
                    self.add_floating_text(sx, sy - 40.0, -100);
                }
                if finished {
                    self.start_monster_turns();
                }
            }
        }
    }

    fn start_monster_turns(&mut self) {
        self.turn = Turn::Monster(0);
        self.monster_turn_timer = MONSTER_TURN_DELAY;
    }

    fn advance_monster_turn(&mut self, dt: f32) {
        if self.is_over() {
            return;
        }
        let Turn::Monster(i) = self.turn else {
            return;
        };
        self.monster_turn_timer -= dt;
        if self.monster_turn_timer > 0.0 {
            return;
        }

        let occupied: Vec<_> = self.monsters.iter().map(|m| m.position()).collect();
        if let MonsterAction::Attack(damage) =
            self.monsters[i].act(&self.map, self.player.position(), &occupied)
        {
            self.player.take_damage(damage);
            let (px, py) = self.player.position();
            let (sx, sy) = to_screen(px, py, self.cam);
            self.add_floating_text(sx, sy - 40.0, damage);
        }

        if !self.is_over() {
            if i + 1 < self.monsters.len() {
                self.turn = Turn::Monster(i + 1);
                self.monster_turn_timer = MONSTER_TURN_DELAY;
            } else {
                self.turn = Turn::Player;
            }
        }
    }

    //helper function to damage a monster
    fn damage_monster(&mut self, idx: usize, dmg: i32) {
        self.monsters[idx].take_damage(dmg);

        //spawn text
        let (x, y) = self.monsters[idx].position();
        let (sx, sy) = to_screen(x, y, self.cam);
        self.add_floating_text(sx, sy, dmg);

        //kill logic
        if self.monsters[idx].health() <= 0 {
            self.monsters.remove(idx);
            self.player.add_score(50);
        }
    }

    fn add_floating_text(&mut self, x: f32, y: f32, dmg: i32) {
        let id = self.next_text_id;
        self.next_text_id = self.next_text_id.wrapping_add(1);
        self.texts.push(DmgText {
            id,
            x,
            y,
            dmg,
            life: 1.0,
        });
    }
}

#[cfg(test)]
#[path = "../tests/unit/game.rs"]
mod tests;
