use super::*;
use crate::mapas::MAP;

#[test]
fn projection_distinguishes_opposite_sides_of_each_edge() {
    let cam = (320.0, 50.0);
    let (sx, sy) = to_screen(5, 5, cam);
    let center_y = sy - TILE_SIZE.1 * 1.5;
    for (dx, dy, neighbor) in [
        (16.0, 8.0, (6, 5)),
        (-16.0, 8.0, (5, 6)),
        (-16.0, -8.0, (4, 5)),
        (16.0, -8.0, (5, 4)),
    ] {
        assert_eq!(to_tile(sx + dx * 0.99, center_y + dy * 0.99, cam), (5, 5));
        assert_eq!(to_tile(sx + dx * 1.01, center_y + dy * 1.01, cam), neighbor);
    }
    let (origin_x, origin_y) = to_screen(0, 0, cam);
    assert_eq!(
        to_tile(origin_x - 17.0, origin_y - TILE_SIZE.1 * 1.5 - 8.5, cam),
        (-1, 0)
    );
}

#[test]
fn to_tile_maps_floor_centers_to_their_isometric_tiles() {
    let cam = (320.0, 50.0);

    for y in 0..MAP {
        for x in 0..MAP {
            let (sx, sy) = to_screen(x, y, cam);
            assert_eq!(
                to_tile(sx, sy - TILE_SIZE.1 * 1.5, cam),
                (x as isize, y as isize)
            );
        }
    }
}

#[test]
fn to_tile_maps_points_inside_a_floor_tile_to_that_tile() {
    let cam = (320.0, 50.0);
    let offsets = [
        (0.0, 0.0),
        (12.0, 4.0),
        (-12.0, 4.0),
        (0.0, -8.0),
        (0.0, 8.0),
    ];

    for y in 0..MAP {
        for x in 0..MAP {
            let (sx, sy) = to_screen(x, y, cam);
            for (dx, dy) in offsets {
                assert_eq!(
                    to_tile(sx + dx, sy - TILE_SIZE.1 * 1.5 + dy, cam),
                    (x as isize, y as isize)
                );
            }
        }
    }
}
