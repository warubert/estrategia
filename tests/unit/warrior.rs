use super::*;
use crate::mapas::{Mapa1, Tile};

#[test]
fn movement_obeys_timer_and_refuses_occupied_or_wall_tiles() {
    let mut map = Mapa1::new();
    let mut player = Warrior::new((2, 2));
    player.start_movement(vec![(3, 2), (4, 2)]);
    assert_eq!(
        player.advance_movement(0.0, &map, &[]),
        Movement::Stepped {
            position: (3, 2),
            finished: false
        }
    );
    assert_eq!(player.advance_movement(0.01, &map, &[]), Movement::Waiting);
    assert_eq!(
        player.advance_movement(1.0, &map, &[(4, 2)]),
        Movement::Blocked
    );
    assert_eq!(player.position(), (3, 2));
    assert!(player.remaining_path().is_empty());

    map.set_tile((4, 2), Tile::Wall);
    player.start_movement(vec![(4, 2)]);
    assert_eq!(player.advance_movement(0.0, &map, &[]), Movement::Blocked);
    assert_eq!(player.position(), (3, 2));
}

#[test]
fn new_warrior_tracks_health_score_and_starting_position() {
    let mut player = Warrior::new((7, 9));
    assert_eq!(player.position(), (7, 9));
    assert_eq!(player.health(), 100);
    assert_eq!(player.score(), 0);
    assert!(player.remaining_path().is_empty());

    player.take_damage(5);
    player.take_damage(10);
    player.add_score(100);
    player.add_score(50);
    assert_eq!(player.health(), 85);
    assert_eq!(player.score(), 150);
}

#[test]
fn movement_completes_at_destination_and_exposes_remaining_steps() {
    let map = Mapa1::new();
    let mut player = Warrior::new((2, 2));
    player.start_movement(vec![(3, 2), (4, 2)]);
    assert_eq!(
        player.remaining_path().iter().copied().collect::<Vec<_>>(),
        vec![(3, 2), (4, 2)]
    );
    assert_eq!(
        player.advance_movement(0.0, &map, &[]),
        Movement::Stepped {
            position: (3, 2),
            finished: false
        }
    );
    assert_eq!(
        player.advance_movement(PLAYER_STEP_DELAY, &map, &[]),
        Movement::Stepped {
            position: (4, 2),
            finished: true
        }
    );
    assert_eq!(player.position(), (4, 2));
    assert!(player.remaining_path().is_empty());
    assert_eq!(player.advance_movement(0.0, &map, &[]), Movement::Blocked);
}

#[test]
fn restarting_movement_replaces_path_and_resets_step_timer() {
    let map = Mapa1::new();
    let mut player = Warrior::new((2, 2));
    player.start_movement(vec![(3, 2), (4, 2)]);
    player.advance_movement(0.0, &map, &[]);
    player.start_movement(vec![(3, 3)]);
    assert_eq!(
        player.advance_movement(0.0, &map, &[]),
        Movement::Stepped {
            position: (3, 3),
            finished: true
        }
    );
}
