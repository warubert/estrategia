mod geometry;
mod interface;
mod mapa1;

pub use geometry::{dist, to_screen, to_tile};
pub use interface::{GameMap, Position, Tile};
pub use mapa1::Mapa1;

pub const MAP: usize = 20;
pub const TILE_SIZE: (f32, f32) = (32.0, 16.0);
