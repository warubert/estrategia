use super::*;
use crate::mapas::{Mapa1, Tile};

#[test]
fn monster_attacks_or_moves_or_passes_once() {
    let map = Mapa1::new();
    let mut monster = Goblin::new((4, 2), 30);
    assert_eq!(monster.act(&map, (3, 2), &[]), MonsterAction::Attack(5));
    assert_eq!(monster.position(), (4, 2));
    assert_eq!(monster.act(&map, (2, 2), &[(3, 2)]), MonsterAction::Pass);
    assert_eq!(monster.position(), (4, 2));
    assert_eq!(monster.act(&map, (2, 2), &[]), MonsterAction::Move);
    assert_eq!(monster.position(), (3, 2));
}

#[test]
fn goblin_only_attacks_orthogonally_adjacent_player() {
    let map = Mapa1::new();
    for player in [(3, 2), (5, 2), (4, 1), (4, 3)] {
        let mut goblin = Goblin::new((4, 2), 30);
        assert_eq!(goblin.act(&map, player, &[]), MonsterAction::Attack(5));
        assert_eq!(goblin.position(), (4, 2));
    }

    let mut goblin = Goblin::new((4, 2), 30);
    assert_eq!(goblin.act(&map, (3, 3), &[]), MonsterAction::Move);
    assert_eq!(dist(goblin.position(), (3, 3)), 1);
}

#[test]
fn enclosed_goblin_passes_and_damage_accumulates() {
    let mut map = Mapa1::new();
    for position in [(3, 2), (5, 2), (4, 1), (4, 3)] {
        map.set_tile(position, Tile::Wall);
    }
    let mut goblin = Goblin::new((4, 2), 30);
    goblin.take_damage(10);
    goblin.take_damage(15);
    assert_eq!(goblin.health(), 5);
    assert_eq!(goblin.act(&map, (2, 2), &[]), MonsterAction::Pass);
    assert_eq!(goblin.position(), (4, 2));
}
