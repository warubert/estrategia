use super::*;

#[test]
fn map_rejects_out_of_bounds_walls_and_blocked_destinations() {
    let map = Mapa1::new();
    assert!(!map.is_walkable((MAP, 0)));
    assert!(!map.is_walkable((0, 0)));
    assert!(map.path((MAP, 2), (2, 2), &[]).is_empty());
    assert!(map.path((2, 2), (MAP, 2), &[]).is_empty());
    assert!(map.path((2, 2), (3, 2), &[(3, 2)]).is_empty());
    let path = map.path((2, 2), (4, 2), &[(3, 2)]);
    assert_eq!(path.last(), Some(&(4, 2)));
    assert!(!path.contains(&(3, 2)));
}

#[test]
fn bfs_never_includes_wall_tiles() {
    let mut map = [[Tile::Floor; MAP]; MAP];
    map[1][2] = Tile::Wall;
    map[2][2] = Tile::Wall;
    map[3][2] = Tile::Wall;

    let path = bfs(&map, (1, 2), (3, 2), &[]);

    assert!(!path.is_empty());
    assert!(path.iter().all(|&(x, y)| map[y][x] == Tile::Floor));
    assert_eq!(path.last(), Some(&(3, 2)));
}

#[test]
fn initial_map_has_impassable_border_and_obstacles() {
    let map = Mapa1::new();
    for i in 0..MAP {
        for position in [(i, 0), (i, MAP - 1), (0, i), (MAP - 1, i)] {
            assert!(!map.is_walkable(position));
        }
    }
    for position in [(5, 5), (6, 5), (12, 10)] {
        assert!(!map.is_walkable(position));
    }
    assert!(map.is_walkable((2, 2)));
}

#[test]
fn paths_are_orthogonal_shortest_routes_and_respect_obstacles() {
    let map = Mapa1::new();
    for (start, goal, blocked, expected_length) in [
        ((2, 2), (4, 2), vec![], 2),
        ((2, 2), (4, 2), vec![(3, 2)], 4),
        ((4, 5), (7, 5), vec![], 5),
        ((2, 2), (2, 2), vec![], 0),
    ] {
        let path = map.path(start, goal, &blocked);
        assert_eq!(path.len(), expected_length);
        let mut previous = start;
        for &step in &path {
            assert!(map.is_walkable(step));
            assert!(!blocked.contains(&step));
            assert_eq!(crate::mapas::dist(previous, step), 1);
            previous = step;
        }
        assert_eq!(previous, goal);
    }
}

#[test]
fn completely_surrounded_goal_is_unreachable() {
    let mut map = Mapa1::new();
    for position in [(7, 8), (9, 8), (8, 7), (8, 9)] {
        map.set_tile(position, Tile::Wall);
    }
    assert!(map.path((2, 2), (8, 8), &[]).is_empty());
}
