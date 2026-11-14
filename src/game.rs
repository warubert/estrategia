use crate::entities::DmgText;
use crate::mapas::{GameMap, MAP, Mapa1, Position, Tile, to_screen, to_tile};
use crate::monstros::{Goblin, MonsterAction, MonsterBehavior};
use crate::personagens::{Movement, PlayerActions, Warrior};
use crate::renderer::{draw_floor, draw_stickman, draw_walls};
use macroquad::prelude::*;

const MONSTER_TURN_DELAY: f32 = 0.3;

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
    gold: Vec<(usize, usize)>,
}

impl Game {
    pub fn new() -> Self {
        Self::with_camera((screen_width() / 2.0, 50.0))
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
            gold: vec![(3, 3), (10, 2), (16, 5), (6, 14), (17, 17)],
        }
    }
}

impl<M: GameMap, P: PlayerActions, N: MonsterBehavior> Game<M, P, N> {
    pub fn player(&self) -> &P {
        &self.player
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
            Turn::Player => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mx, my) = mouse_position();
                    let (tx, ty) = to_tile(mx, my, self.cam);
                    if tx >= 0 && ty >= 0 && tx < MAP as isize && ty < MAP as isize {
                        self.player_action((tx as usize, ty as usize));
                    }
                }
            }
            Turn::PlayerMoving => self.advance_player_movement(dt),
            Turn::Monster(_) => self.advance_monster_turn(dt),
        }

        self.is_over()
    }

    fn is_over(&self) -> bool {
        self.player.health() <= 0 || self.monsters.is_empty()
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

    fn player_action(&mut self, target: (usize, usize)) {
        if self.turn != Turn::Player || self.is_over() {
            return;
        }
        let path = self.player_path(target);
        let Some(&(nx, ny)) = path.first() else {
            return;
        };

        if let Some(i) = self.monsters.iter().position(|m| m.position() == (nx, ny)) {
            self.damage_monster(i, 10);
            if !self.is_over() {
                self.start_monster_turns();
            }
        } else {
            self.player.start_movement(path);
            self.turn = Turn::PlayerMoving;
            self.advance_player_movement(0.0);
        }
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
                    self.texts.push(DmgText {
                        x: sx,
                        y: sy - 40.,
                        dmg: -100,
                        life: 1.0,
                    });
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
            self.texts.push(DmgText {
                x: sx,
                y: sy - 40.,
                dmg: damage,
                life: 1.0,
            });
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
        self.texts.push(DmgText {
            x: sx,
            y: sy,
            dmg,
            life: 1.0,
        });

        //kill logic
        if self.monsters[idx].health() <= 0 {
            self.monsters.remove(idx);
            self.player.add_score(50);
        }
    }

    pub fn draw(&self) {
        let (mx, my) = mouse_position();
        let hovered_tile = to_tile(mx, my, self.cam);
        let preview_path: Vec<(usize, usize)> = if self.turn == Turn::PlayerMoving {
            self.player.remaining_path().iter().copied().collect()
        } else if self.turn == Turn::Player
            && !self.is_over()
            && hovered_tile.0 >= 0
            && hovered_tile.1 >= 0
            && hovered_tile.0 < MAP as isize
            && hovered_tile.1 < MAP as isize
        {
            self.player_path((hovered_tile.0 as usize, hovered_tile.1 as usize))
        } else {
            vec![]
        };

        for depth in 0..(MAP * 2 - 1) {
            for y in 0..MAP {
                if y > depth {
                    continue;
                }
                let x = depth - y;
                if x >= MAP {
                    continue;
                }

                if self.map.tile((x, y)) == Some(Tile::Wall) {
                    draw_walls(x, y, self.cam);
                } else {
                    draw_floor(x, y, self.cam, hovered_tile == (x as isize, y as isize));
                    let (sx, sy) = to_screen(x, y, self.cam);

                    if self.gold.contains(&(x, y)) {
                        draw_circle(sx, sy - 24., 6., GOLD);
                    }
                }

                if preview_path.contains(&(x, y)) {
                    let (sx, sy) = to_screen(x, y, self.cam);
                    draw_circle(sx, sy - 24., 4., GOLD);
                }

                if (x, y) == self.player.position() {
                    draw_stickman(x, y, self.cam, false);
                }

                for monster in &self.monsters {
                    if monster.position() == (x, y) {
                        draw_stickman(x, y, self.cam, true);
                    }
                }
            }
        }

        //draw floating texts
        for text in &self.texts {
            if text.dmg < 0 {
                draw_text(format!("+{}", -text.dmg), text.x, text.y, 20., GREEN);
            } else {
                draw_text(format!("-{}", text.dmg), text.x, text.y, 20., RED);
            }
        }

        //HUD
        draw_text(
            format!("HP: {}", self.player.health()),
            20.,
            screen_height() - 40.,
            30.,
            BLACK,
        );
        draw_text(
            format!("Score: {}", self.player.score()),
            20.,
            screen_height() - 70.,
            30.,
            BLACK,
        );
        let turn_text = match self.turn {
            Turn::Player => "Turno: jogador".to_owned(),
            Turn::PlayerMoving => "Turno: jogador (movendo)".to_owned(),
            Turn::Monster(i) => format!("Turno: monstro {}/{}", i + 1, self.monsters.len()),
        };
        draw_text(turn_text, 20., screen_height() - 100., 30., BLACK);
    }
}

#[cfg(test)]
#[path = "../tests/unit/game.rs"]
mod tests;
