use super::{GameMap, MAP, Position, Tile};
use std::collections::VecDeque;

pub struct Mapa1 {
    tiles: [[Tile; MAP]; MAP],
}

impl Mapa1 {
    pub fn new() -> Self {
        let mut tiles = [[Tile::Floor; MAP]; MAP];
        for i in 0..MAP {
            tiles[0][i] = Tile::Wall;
            tiles[MAP - 1][i] = Tile::Wall;
            tiles[i][0] = Tile::Wall;
            tiles[i][MAP - 1] = Tile::Wall;
        }
        for (x, y) in [(5, 5), (6, 5), (12, 10)] {
            tiles[y][x] = Tile::Wall;
        }
        Self { tiles }
    }

    #[cfg(test)]
    pub fn set_tile(&mut self, (x, y): Position, tile: Tile) {
        self.tiles[y][x] = tile;
    }
}

impl GameMap for Mapa1 {
    fn tile(&self, (x, y): Position) -> Option<Tile> {
        self.tiles.get(y).and_then(|row| row.get(x)).copied()
    }

    fn path(&self, start: Position, goal: Position, blocked: &[Position]) -> Vec<Position> {
        if !self.is_walkable(start) || !self.is_walkable(goal) || blocked.contains(&goal) {
            return vec![];
        }
        bfs(&self.tiles, start, goal, blocked)
    }
}

//Pathfinder algorithm
fn bfs(
    map: &[[Tile; MAP]; MAP],
    start: (usize, usize),
    goal: (usize, usize),
    blocked: &[Position],
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

            if nx < MAP
                && ny < MAP
                && !visited[ny][nx]
                && map[ny][nx] == Tile::Floor
                && !blocked.contains(&(nx, ny))
            {
                visited[ny][nx] = true;
                parent[ny][nx] = Some(curr);
                q.push_back((nx, ny));
            }
        }
    }

    vec![]
}

#[cfg(test)]
#[path = "../../../tests/unit/mapa1.rs"]
mod tests;
