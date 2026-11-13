use macroquad::prelude::*;
use std::collections::VecDeque;

pub const MAP: usize = 20;
pub const TILE_SIZE: (f32, f32) = (32.0, 16.0);

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    Wall,
    Floor,
}

pub fn to_screen(x: usize, y: usize, cam: (f32, f32)) -> (f32, f32) {
    (
        (x as f32 - y as f32) * TILE_SIZE.0 + cam.0,
        (x as f32 + y as f32) * TILE_SIZE.1 + cam.1,
    )
}

pub fn to_tile(sx: f32, sy: f32, cam: (f32, f32)) -> (isize, isize) {
    let (ax, ay) = (sx - cam.0, sy - cam.1 + TILE_SIZE.1 * 1.5);

    (
        ((ax / TILE_SIZE.0 + ay / TILE_SIZE.1) / 2.).round() as isize,
        ((ay / TILE_SIZE.1 - ax / TILE_SIZE.0) / 2.).round() as isize,
    )
}

//calculate Manhattan distance between two points
pub fn dist(p1: (usize, usize), p2: (usize, usize)) -> i32 {
    (p1.0 as i32 - p2.0 as i32).abs() + (p1.1 as i32 - p2.1 as i32).abs()
}

//Pathfinder algorithm
pub fn bfs(
    map: &[[Tile; MAP]; MAP],
    start: (usize, usize),
    goal: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut q = VecDeque::from([start]);
    let mut visited = [[false; MAP]; MAP];
    visited[start.1][start.0] = true;

    let mut parent = [[None; MAP]; MAP];

    while let Some(curr) = q.pop_front() {
        if curr == goal {
            let mut path = vec![];
            let mut c = goal;
            while c != start {
                path.push(c);
                c = parent[c.1][c.0].unwrap();
            }
            path.reverse();
            return path;
        }

        //check close
        for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
            let (nx, ny) = ((curr.0 as i32 + dx) as usize, (curr.1 as i32 + dy) as usize);

            if nx < MAP && ny < MAP && !visited[ny][nx] && map[ny][nx] == Tile::Floor {
                visited[ny][nx] = true;
                parent[ny][nx] = Some(curr);
                q.push_back((nx, ny));
            }
        }
    }

    vec![]
}

pub fn draw_walls(x: usize, y: usize, cam: (f32, f32)) {
    let (sx, sy) = to_screen(x, y, cam);

    let v = [
        vec2(sx, sy - 72.),
        vec2(sx + 32., sy - 56.),
        vec2(sx, sy - 40.),
        vec2(sx - 32., sy - 56.),
        vec2(sx + 32., sy - 24.),
        vec2(sx, sy - 8.),
        vec2(sx - 32., sy - 24.),
    ];

    let colors = [
        Color::new(0.8, 0.8, 0.8, 1.0),
        Color::new(0.5, 0.5, 0.5, 1.0),
        Color::new(0.6, 0.6, 0.6, 1.0),
    ];

    //draw faces
    draw_triangle(v[0], v[1], v[2], colors[0]);
    draw_triangle(v[0], v[2], v[3], colors[0]);
    draw_triangle(v[1], v[4], v[5], colors[1]);
    draw_triangle(v[1], v[5], v[2], colors[1]);
    draw_triangle(v[3], v[2], v[5], colors[2]);
    draw_triangle(v[3], v[5], v[6], colors[2]);

    //draw outlines
    for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 0), (1, 4), (2, 5), (3, 6)] {
        draw_line(v[a].x, v[a].y, v[b].x, v[b].y, 1.0, BLACK);
    }
}

pub fn draw_floor(x: usize, y: usize, cam: (f32, f32), highlighted: bool) {
    let (sx, sy) = to_screen(x, y, cam);
    let top = vec2(sx, sy - 40.);
    let right = vec2(sx + 32., sy - 24.);
    let bottom = vec2(sx, sy - 8.);
    let left = vec2(sx - 32., sy - 24.);
    let border = Color::new(0.65, 0.65, 0.65, 1.0);

    if highlighted {
        let highlight = Color::new(0.65, 0.85, 1.0, 1.0);
        draw_triangle(top, right, bottom, highlight);
        draw_triangle(top, bottom, left, highlight);
    }

    for (a, b) in [(top, right), (right, bottom), (bottom, left), (left, top)] {
        draw_line(a.x, a.y, b.x, b.y, 1.0, border);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bfs_never_includes_wall_tiles() {
        let mut map = [[Tile::Floor; MAP]; MAP];
        map[1][2] = Tile::Wall;
        map[2][2] = Tile::Wall;
        map[3][2] = Tile::Wall;

        let path = bfs(&map, (1, 2), (3, 2));

        assert!(!path.is_empty());
        assert!(path.iter().all(|&(x, y)| map[y][x] == Tile::Floor));
        assert_eq!(path.last(), Some(&(3, 2)));
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
}
