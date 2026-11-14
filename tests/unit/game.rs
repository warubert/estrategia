use super::*;
use crate::mapas::dist;

const PLAYER_STEP_DELAY: f32 = 0.15;

struct TestMap;

impl GameMap for TestMap {
    fn tile(&self, (x, y): Position) -> Option<Tile> {
        (x < MAP && y < MAP).then_some(Tile::Floor)
    }

    fn path(&self, start: Position, goal: Position, _blocked: &[Position]) -> Vec<Position> {
        if start == (2, 2) && goal == (3, 2) {
            vec![goal]
        } else {
            vec![]
        }
    }
}

#[test]
fn game_can_use_an_alternative_map_implementation() {
    let mut game = Game {
        map: TestMap,
        cam: (0.0, 0.0),
        player: Warrior::new((2, 2)),
        turn: Turn::Player,
        monster_turn_timer: 0.0,
        monsters: vec![Goblin::new((8, 8), 30)],
        texts: vec![],
        gold: vec![],
    };
    game.player_action((3, 2));
    assert_eq!(game.player.position(), (3, 2));
    assert_eq!(game.turn, Turn::Monster(0));
}

fn monster(x: usize, y: usize, hp: i32) -> Goblin {
    Goblin::new((x, y), hp)
}

#[test]
fn player_reaches_destination_then_monsters_act_once_in_order() {
    let mut game = Game::with_camera((0.0, 0.0));
    game.monsters = vec![monster(8, 2, 30), monster(12, 4, 30)];
    game.player_action((4, 2));
    assert_eq!(game.player.position(), (3, 2));
    assert_eq!(game.turn, Turn::PlayerMoving);
    assert_eq!(game.player.score(), 0);

    game.player_action((2, 2));
    game.advance_monster_turn(MONSTER_TURN_DELAY);
    assert_eq!(game.player.position(), (3, 2));
    assert_eq!(game.monsters[0].position(), (8, 2));
    game.advance_player_movement(PLAYER_STEP_DELAY / 2.0);
    assert_eq!(game.player.position(), (3, 2));
    game.advance_player_movement(PLAYER_STEP_DELAY);
    assert_eq!(game.player.position(), (4, 2));
    assert_eq!(game.turn, Turn::Monster(0));
    game.advance_monster_turn(MONSTER_TURN_DELAY / 2.0);
    assert_eq!(game.monsters[0].position(), (8, 2));
    game.advance_monster_turn(MONSTER_TURN_DELAY);
    assert_eq!(dist((8, 2), game.monsters[0].position()), 1);
    assert_eq!(game.monsters[1].position(), (12, 4));
    assert_eq!(game.turn, Turn::Monster(1));

    let first = game.monsters[0].position();
    game.advance_monster_turn(MONSTER_TURN_DELAY);
    assert_eq!(game.monsters[0].position(), first);
    assert_eq!(dist((12, 4), game.monsters[1].position()), 1);
    assert_eq!(game.turn, Turn::Player);
    assert_eq!(game.player.position(), (4, 2));
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
    assert_eq!(game.player.position(), (5, 2));
    assert!(game.gold.is_empty());
    assert_eq!(game.player.score(), 200);
    assert_eq!(game.monsters[0].position(), (3, 2));
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
    assert_eq!(game.player.position(), (4, 2));
    assert_eq!(game.monsters[0].health(), 30);
    assert_eq!(game.turn, Turn::Monster(0));
    game.advance_monster_turn(MONSTER_TURN_DELAY);
    assert_eq!(game.player.health(), 95);
    game.player_action((5, 2));
    assert_eq!(game.monsters[0].health(), 20);
    assert_eq!(game.player.position(), (4, 2));
    assert_eq!(game.turn, Turn::Monster(0));
}

#[test]
fn path_is_cancelled_if_a_future_tile_becomes_blocked() {
    let mut game = Game::with_camera((0.0, 0.0));
    game.player_action((4, 2));
    game.map.set_tile((4, 2), Tile::Wall);
    game.advance_player_movement(PLAYER_STEP_DELAY);
    assert_eq!(game.player.position(), (3, 2));
    assert_eq!(game.turn, Turn::Monster(0));
    assert!(game.player.remaining_path().is_empty());
}

#[test]
fn invalid_clicks_do_not_consume_the_turn() {
    let mut game = Game::with_camera((0.0, 0.0));
    for target in [(2, 2), (0, 2), (MAP, 2), (5, 5)] {
        game.player_action(target);
        assert_eq!(game.turn, Turn::Player);
        assert_eq!(game.player.position(), (2, 2));
    }
    for (x, y) in [(3, 2), (1, 2), (2, 1), (2, 3)] {
        game.map.set_tile((x, y), Tile::Wall);
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
    assert_eq!(game.player.score(), 100);
    assert_eq!(game.turn, Turn::Monster(0));
    game.advance_monster_turn(MONSTER_TURN_DELAY);
    assert_eq!(game.player.health(), 95);
    assert_eq!(game.turn, Turn::Player);

    game.player_action((4, 2));
    assert_eq!(game.player.position(), (3, 2));
    assert_eq!(game.monsters[0].health(), 20);
    assert_eq!(game.turn, Turn::Monster(0));
}

#[test]
fn defeated_monsters_do_not_take_a_turn_and_victory_is_immediate() {
    let mut game = Game::with_camera((0.0, 0.0));
    game.monsters = vec![monster(3, 2, 10), monster(2, 3, 10)];
    game.player_action((3, 2));
    assert_eq!(game.player.score(), 50);
    assert_eq!(game.monsters.len(), 1);
    assert_eq!(game.turn, Turn::Monster(0));
    game.advance_monster_turn(MONSTER_TURN_DELAY);
    assert_eq!(game.player.health(), 95);
    assert_eq!(game.turn, Turn::Player);
    game.player_action((2, 3));
    assert!(game.is_over());
    assert_eq!(game.player.score(), 100);
    assert_eq!(game.player.health(), 95);
}

#[test]
fn defeat_stops_remaining_monster_turns() {
    let mut game = Game::with_camera((0.0, 0.0));
    game.player.take_damage(95);
    game.monsters = vec![monster(4, 2, 30), monster(3, 3, 30)];
    game.player_action((3, 2));
    game.advance_monster_turn(MONSTER_TURN_DELAY);
    assert!(game.is_over());
    game.advance_monster_turn(MONSTER_TURN_DELAY);
    assert_eq!(game.player.health(), 0);
    assert_eq!(game.monsters[1].position(), (3, 3));
}

#[test]
fn blocked_monster_passes_without_overlapping_another() {
    let mut game = Game::with_camera((0.0, 0.0));
    game.monsters = vec![monster(4, 2, 30), monster(5, 2, 30)];
    game.player_action((3, 2));
    game.advance_monster_turn(MONSTER_TURN_DELAY);
    assert_eq!(game.player.health(), 95);
    game.advance_monster_turn(MONSTER_TURN_DELAY);
    assert_eq!(game.monsters[1].position(), (5, 2));
    assert_eq!(game.turn, Turn::Player);
}

#[test]
fn three_attacks_take_three_turns_and_award_kill_points_once() {
    let mut game = Game::with_camera((0.0, 0.0));
    game.monsters = vec![monster(3, 2, 30)];

    for expected_health in [20, 10] {
        game.player_action((3, 2));
        assert_eq!(game.monsters[0].health(), expected_health);
        assert_eq!(game.player.position(), (2, 2));
        assert_eq!(game.turn, Turn::Monster(0));
        game.advance_monster_turn(MONSTER_TURN_DELAY);
        assert_eq!(game.turn, Turn::Player);
    }
    assert_eq!(game.player.health(), 90);
    game.player_action((3, 2));
    assert!(game.is_over());
    assert!(game.monsters.is_empty());
    assert_eq!(game.player.score(), 50);
    game.player_action((2, 3));
    game.advance_monster_turn(MONSTER_TURN_DELAY);
    assert_eq!(game.player.score(), 50);
    assert_eq!(game.player.health(), 90);
}

#[test]
fn revisiting_a_collected_coin_does_not_award_more_points() {
    let mut game = Game::with_camera((0.0, 0.0));
    game.monsters = vec![monster(15, 15, 30)];
    game.gold = vec![(3, 2)];
    game.player_action((3, 2));
    assert_eq!(game.player.score(), 100);
    assert_eq!(game.texts.len(), 1);
    assert_eq!(game.texts[0].dmg, -100);
    game.advance_monster_turn(MONSTER_TURN_DELAY);
    game.player_action((2, 2));
    game.advance_monster_turn(MONSTER_TURN_DELAY);
    game.player_action((3, 2));
    assert_eq!(game.player.position(), (3, 2));
    assert_eq!(game.player.score(), 100);
    assert!(game.gold.is_empty());
    assert_eq!(game.texts.len(), 1);
}

#[test]
fn preview_and_movement_stop_before_distant_monster() {
    let mut game = Game::with_camera((0.0, 0.0));
    game.monsters = vec![monster(6, 2, 30)];
    let path = game.player_path((6, 2));
    assert_eq!(path, vec![(3, 2), (4, 2), (5, 2)]);
    game.player_action((6, 2));
    for _ in 0..path.len() - 1 {
        game.advance_player_movement(PLAYER_STEP_DELAY);
    }
    assert_eq!(game.player.position(), (5, 2));
    assert_eq!(game.monsters[0].health(), 30);
    assert_eq!(game.turn, Turn::Monster(0));
    assert!(game.player.remaining_path().is_empty());
}

#[test]
fn newly_occupied_next_step_cancels_movement_without_attacking() {
    let mut game = Game::with_camera((0.0, 0.0));
    game.monsters = vec![monster(8, 2, 30)];
    game.player_action((5, 2));
    assert_eq!(game.player.position(), (3, 2));
    game.monsters.push(monster(4, 2, 30));
    game.advance_player_movement(PLAYER_STEP_DELAY);
    assert_eq!(game.player.position(), (3, 2));
    assert_eq!(game.monsters[1].health(), 30);
    assert!(game.player.remaining_path().is_empty());
    assert_eq!(game.turn, Turn::Monster(0));
}

#[test]
fn dead_player_cannot_start_movement_or_attack() {
    let mut game = Game::with_camera((0.0, 0.0));
    game.monsters = vec![monster(3, 2, 30)];
    game.player.take_damage(100);
    game.player_action((3, 2));
    game.player_action((2, 3));
    assert_eq!(game.monsters[0].health(), 30);
    assert_eq!(game.player.position(), (2, 2));
    assert_eq!(game.turn, Turn::Player);
    assert!(game.is_over());
}
