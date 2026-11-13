use crate::entities::{DmgText, Monster};
use crate::map::{MAP, Tile, bfs, dist, draw_floor, draw_walls, to_screen, to_tile};
use crate::renderer::draw_stickman;
use macroquad::prelude::*;
use std::collections::VecDeque;

const MONSTER_TURN_DELAY: f32 = 0.3;
const PLAYER_STEP_DELAY: f32 = 0.15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Turn {
    Player,
    PlayerMoving,
    Monster(usize),
}

pub struct Game {
    map: [[Tile; MAP]; MAP],
    cam: (f32, f32),
    px: usize,
    py: usize,
    turn: Turn,
    monster_turn_timer: f32,
    player_step_timer: f32,
    player_path: VecDeque<(usize, usize)>,
    monsters: Vec<Monster>,
    texts: Vec<DmgText>,
    pub hp: i32,
    gold: Vec<(usize, usize)>,
    pub score: i32,
}

impl Game {
    pub fn new() -> Self {
        Self::with_camera((screen_width() / 2.0, 50.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monster(x: usize, y: usize, hp: i32) -> Monster {
        Monster { x, y, hp }
    }

    #[test]
    fn player_reaches_destination_then_monsters_act_once_in_order() {
        let mut game = Game::with_camera((0.0, 0.0));
        game.monsters = vec![monster(8, 2, 30), monster(12, 4, 30)];
        game.player_action((4, 2));
        assert_eq!((game.px, game.py), (3, 2));
        assert_eq!(game.turn, Turn::PlayerMoving);
        assert_eq!(game.score, 0);

        game.player_action((2, 2));
        game.advance_monster_turn(MONSTER_TURN_DELAY);
        assert_eq!((game.px, game.py), (3, 2));
        assert_eq!((game.monsters[0].x, game.monsters[0].y), (8, 2));
        game.advance_player_movement(PLAYER_STEP_DELAY / 2.0);
        assert_eq!((game.px, game.py), (3, 2));
        game.advance_player_movement(PLAYER_STEP_DELAY);
        assert_eq!((game.px, game.py), (4, 2));
        assert_eq!(game.turn, Turn::Monster(0));
        game.advance_monster_turn(MONSTER_TURN_DELAY / 2.0);
        assert_eq!((game.monsters[0].x, game.monsters[0].y), (8, 2));
        game.advance_monster_turn(MONSTER_TURN_DELAY);
        assert_eq!(dist((8, 2), (game.monsters[0].x, game.monsters[0].y)), 1);
        assert_eq!((game.monsters[1].x, game.monsters[1].y), (12, 4));
        assert_eq!(game.turn, Turn::Monster(1));

        let first = (game.monsters[0].x, game.monsters[0].y);
        game.advance_monster_turn(MONSTER_TURN_DELAY);
        assert_eq!((game.monsters[0].x, game.monsters[0].y), first);
        assert_eq!(dist((12, 4), (game.monsters[1].x, game.monsters[1].y)), 1);
        assert_eq!(game.turn, Turn::Player);
        assert_eq!((game.px, game.py), (4, 2));
    }

    #[test]
    fn path_detours_around_monsters_and_collects_gold_on_the_way() {
        let mut game = Game::with_camera((0.0, 0.0));
        game.monsters = vec![monster(3, 2, 30)];
        let path = game.player_path((5, 2));
        assert!(!path.is_empty());
        assert!(!path.contains(&(3, 2)));
        game.gold = vec![path[0], *path.last().unwrap()];
        game.player_action((5, 2));
        while game.turn == Turn::PlayerMoving {
            game.advance_player_movement(PLAYER_STEP_DELAY);
        }
        assert_eq!((game.px, game.py), (5, 2));
        assert!(game.gold.is_empty());
        assert_eq!(game.score, 200);
        assert_eq!((game.monsters[0].x, game.monsters[0].y), (3, 2));
        assert_eq!(game.turn, Turn::Monster(0));
    }

    #[test]
    fn distant_monster_requires_separate_movement_and_attack_actions() {
        let mut game = Game::with_camera((0.0, 0.0));
        game.monsters = vec![monster(5, 2, 30)];
        game.player_action((5, 2));
        while game.turn == Turn::PlayerMoving {
            game.advance_player_movement(PLAYER_STEP_DELAY);
        }
        assert_eq!((game.px, game.py), (4, 2));
        assert_eq!(game.monsters[0].hp, 30);
        assert_eq!(game.turn, Turn::Monster(0));
        game.advance_monster_turn(MONSTER_TURN_DELAY);
        assert_eq!(game.hp, 95);
        game.player_action((5, 2));
        assert_eq!(game.monsters[0].hp, 20);
        assert_eq!((game.px, game.py), (4, 2));
        assert_eq!(game.turn, Turn::Monster(0));
    }

    #[test]
    fn path_is_cancelled_if_a_future_tile_becomes_blocked() {
        let mut game = Game::with_camera((0.0, 0.0));
        game.player_action((4, 2));
        game.map[2][4] = Tile::Wall;
        game.advance_player_movement(PLAYER_STEP_DELAY);
        assert_eq!((game.px, game.py), (3, 2));
        assert_eq!(game.turn, Turn::Monster(0));
        assert!(game.player_path.is_empty());
    }

    #[test]
    fn invalid_clicks_do_not_consume_the_turn() {
        let mut game = Game::with_camera((0.0, 0.0));
        for target in [(2, 2), (0, 2), (MAP, 2), (5, 5)] {
            game.player_action(target);
            assert_eq!(game.turn, Turn::Player);
            assert_eq!((game.px, game.py), (2, 2));
        }
        for (x, y) in [(3, 2), (1, 2), (2, 1), (2, 3)] {
            game.map[y][x] = Tile::Wall;
        }
        game.player_action((4, 2));
        assert_eq!(game.turn, Turn::Player);
    }

    #[test]
    fn player_collects_gold_and_attack_consumes_a_turn() {
        let mut game = Game::with_camera((0.0, 0.0));
        game.gold = vec![(3, 2)];
        game.monsters = vec![monster(4, 2, 30)];
        game.player_action((3, 2));
        assert!(game.gold.is_empty());
        assert_eq!(game.score, 100);
        assert_eq!(game.turn, Turn::Monster(0));
        game.advance_monster_turn(MONSTER_TURN_DELAY);
        assert_eq!(game.hp, 95);
        assert_eq!(game.turn, Turn::Player);

        game.player_action((4, 2));
        assert_eq!((game.px, game.py), (3, 2));
        assert_eq!(game.monsters[0].hp, 20);
        assert_eq!(game.turn, Turn::Monster(0));
    }

    #[test]
    fn defeated_monsters_do_not_take_a_turn_and_victory_is_immediate() {
        let mut game = Game::with_camera((0.0, 0.0));
        game.monsters = vec![monster(3, 2, 10), monster(2, 3, 10)];
        game.player_action((3, 2));
        assert_eq!(game.score, 50);
        assert_eq!(game.monsters.len(), 1);
        assert_eq!(game.turn, Turn::Monster(0));
        game.advance_monster_turn(MONSTER_TURN_DELAY);
        assert_eq!(game.hp, 95);
        assert_eq!(game.turn, Turn::Player);
        game.player_action((2, 3));
        assert!(game.is_over());
        assert_eq!(game.score, 100);
        assert_eq!(game.hp, 95);
    }

    #[test]
    fn defeat_stops_remaining_monster_turns() {
        let mut game = Game::with_camera((0.0, 0.0));
        game.hp = 5;
        game.monsters = vec![monster(4, 2, 30), monster(3, 3, 30)];
        game.player_action((3, 2));
        game.advance_monster_turn(MONSTER_TURN_DELAY);
        assert!(game.is_over());
        game.advance_monster_turn(MONSTER_TURN_DELAY);
        assert_eq!(game.hp, 0);
        assert_eq!((game.monsters[1].x, game.monsters[1].y), (3, 3));
    }

    #[test]
    fn blocked_monster_passes_without_overlapping_another() {
        let mut game = Game::with_camera((0.0, 0.0));
        game.monsters = vec![monster(4, 2, 30), monster(5, 2, 30)];
        game.player_action((3, 2));
        game.advance_monster_turn(MONSTER_TURN_DELAY);
        assert_eq!(game.hp, 95);
        game.advance_monster_turn(MONSTER_TURN_DELAY);
        assert_eq!((game.monsters[1].x, game.monsters[1].y), (5, 2));
        assert_eq!(game.turn, Turn::Player);
    }
}

impl Game {
    fn with_camera(cam: (f32, f32)) -> Self {
        let mut map = [[Tile::Floor; MAP]; MAP];

        for i in 0..MAP {
            map[0][i] = Tile::Wall;
            map[MAP - 1][i] = Tile::Wall;
            map[i][0] = Tile::Wall;
            map[i][MAP - 1] = Tile::Wall;
        }

        //add obstacles
        for (x, y) in [(5, 5), (6, 5), (12, 10)] {
            map[y][x] = Tile::Wall;
        }

        Game {
            map,
            cam,
            px: 2,
            py: 2,
            turn: Turn::Player,
            monster_turn_timer: 0.0,
            player_step_timer: 0.0,
            player_path: VecDeque::new(),
            monsters: vec![
                Monster { x: 8, y: 8, hp: 30 },
                Monster {
                    x: 12,
                    y: 4,
                    hp: 30,
                },
                Monster {
                    x: 15,
                    y: 12,
                    hp: 30,
                },
            ],
            texts: vec![],
            hp: 100,
            score: 0,
            gold: vec![(3, 3), (10, 2), (16, 5), (6, 14), (17, 17)],
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
        self.hp <= 0 || self.monsters.is_empty()
    }

    fn player_path(&self, target: (usize, usize)) -> Vec<(usize, usize)> {
        if target.0 >= MAP || target.1 >= MAP || self.map[target.1][target.0] != Tile::Floor {
            return vec![];
        }
        let mut map = self.map;
        for monster in &self.monsters {
            if (monster.x, monster.y) != target {
                map[monster.y][monster.x] = Tile::Wall;
            }
        }
        let mut path = bfs(&map, (self.px, self.py), target);
        if path.len() > 1 && self.monsters.iter().any(|m| (m.x, m.y) == target) {
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

        if let Some(i) = self.monsters.iter().position(|m| (m.x, m.y) == (nx, ny)) {
            self.damage_monster(i, 10);
            if !self.is_over() {
                self.start_monster_turns();
            }
        } else {
            self.player_path = path.into();
            self.turn = Turn::PlayerMoving;
            self.player_step_timer = 0.0;
            self.advance_player_movement(0.0);
        }
    }

    fn advance_player_movement(&mut self, dt: f32) {
        if self.turn != Turn::PlayerMoving || self.is_over() {
            return;
        }
        self.player_step_timer -= dt;
        if self.player_step_timer > 0.0 {
            return;
        }

        let Some((nx, ny)) = self.player_path.pop_front() else {
            self.start_monster_turns();
            return;
        };
        if nx >= MAP
            || ny >= MAP
            || self.map[ny][nx] != Tile::Floor
            || self.monsters.iter().any(|m| (m.x, m.y) == (nx, ny))
        {
            self.player_path.clear();
            self.start_monster_turns();
            return;
        }
        self.px = nx;
        self.py = ny;
        if let Some(i) = self.gold.iter().position(|&g| g == (nx, ny)) {
            self.gold.remove(i);
            self.score += 100;
            let (sx, sy) = to_screen(nx, ny, self.cam);
            self.texts.push(DmgText {
                x: sx,
                y: sy - 40.,
                dmg: -100,
                life: 1.0,
            });
        }

        if self.player_path.is_empty() {
            self.start_monster_turns();
        } else {
            self.player_step_timer = PLAYER_STEP_DELAY;
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

        let (mx, my) = (self.monsters[i].x, self.monsters[i].y);
        if dist((mx, my), (self.px, self.py)) == 1 {
            self.hp -= 5;
            let (sx, sy) = to_screen(self.px, self.py, self.cam);
            self.texts.push(DmgText {
                x: sx,
                y: sy - 40.,
                dmg: 5,
                life: 1.0,
            });
        } else {
            let path = bfs(&self.map, (mx, my), (self.px, self.py));
            if let Some(&next) = path.first() {
                let occupied = self.monsters.iter().any(|m| (m.x, m.y) == next);
                if !occupied && next != (self.px, self.py) {
                    self.monsters[i].x = next.0;
                    self.monsters[i].y = next.1;
                }
            }
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
        self.monsters[idx].hp -= dmg;

        //spawn text
        let (sx, sy) = to_screen(self.monsters[idx].x, self.monsters[idx].y, self.cam);
        self.texts.push(DmgText {
            x: sx,
            y: sy,
            dmg,
            life: 1.0,
        });

        //kill logic
        if self.monsters[idx].hp <= 0 {
            self.monsters.remove(idx);
            self.score += 50;
        }
    }

    pub fn draw(&self) {
        let (mx, my) = mouse_position();
        let hovered_tile = to_tile(mx, my, self.cam);
        let preview_path: Vec<(usize, usize)> = if self.turn == Turn::PlayerMoving {
            self.player_path.iter().copied().collect()
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

                if self.map[y][x] == Tile::Wall {
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

                if (x, y) == (self.px, self.py) {
                    draw_stickman(self.px, self.py, self.cam, false);
                }

                for monster in &self.monsters {
                    if (monster.x, monster.y) == (x, y) {
                        draw_stickman(monster.x, monster.y, self.cam, true);
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
            format!("HP: {}", self.hp),
            20.,
            screen_height() - 40.,
            30.,
            BLACK,
        );
        draw_text(
            format!("Score: {}", self.score),
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
