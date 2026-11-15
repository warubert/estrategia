use crate::game::Game;
use crate::mapas::{GameMap, MAP, Tile, to_screen};
use crate::monstros::MonsterBehavior;
use crate::personagens::PlayerActions;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

const WINDOW_SIZE: (f32, f32) = (1280.0, 720.0);

#[derive(Resource)]
pub struct WorldVisuals {
    pub entity: Entity,
    mesh: Handle<Mesh>,
}

#[derive(Default)]
struct Geometry {
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl Geometry {
    fn polygon(&mut self, points: &[(f32, f32)], depth: f32, color: Color) {
        if points.len() < 3 {
            return;
        }

        let start = self.positions.len() as u32;
        let color = color.to_linear().to_f32_array();
        for &(x, y) in points {
            self.positions
                .push([x - WINDOW_SIZE.0 / 2.0, WINDOW_SIZE.1 / 2.0 - y, depth]);
            self.colors.push(color);
        }

        for index in 1..points.len() - 1 {
            self.indices
                .extend([start, start + index as u32 + 1, start + index as u32]);
        }
    }

    fn line(&mut self, start: (f32, f32), end: (f32, f32), width: f32, depth: f32, color: Color) {
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let length = (dx * dx + dy * dy).sqrt();
        if length == 0.0 {
            return;
        }

        let ox = -dy / length * width / 2.0;
        let oy = dx / length * width / 2.0;
        self.polygon(
            &[
                (start.0 + ox, start.1 + oy),
                (end.0 + ox, end.1 + oy),
                (end.0 - ox, end.1 - oy),
                (start.0 - ox, start.1 - oy),
            ],
            depth,
            color,
        );
    }

    fn circle(&mut self, center: (f32, f32), radius: f32, depth: f32, color: Color, filled: bool) {
        let segments = 16;
        let start = self.positions.len() as u32;
        let color = color.to_linear().to_f32_array();

        if filled {
            self.positions.push([
                center.0 - WINDOW_SIZE.0 / 2.0,
                WINDOW_SIZE.1 / 2.0 - center.1,
                depth,
            ]);
            self.colors.push(color);
        }

        for step in 0..segments {
            let angle = std::f32::consts::TAU * step as f32 / segments as f32;
            self.positions.push([
                center.0 + angle.cos() * radius - WINDOW_SIZE.0 / 2.0,
                WINDOW_SIZE.1 / 2.0 - (center.1 + angle.sin() * radius),
                depth,
            ]);
            self.colors.push(color);
        }

        if filled {
            for index in 0..segments {
                let next = (index + 1) % segments;
                self.indices
                    .extend([start, start + next + 1, start + index + 1]);
            }
        } else {
            for index in 0..segments {
                let next = (index + 1) % segments;
                let a = (
                    center.0
                        + (std::f32::consts::TAU * index as f32 / segments as f32).cos() * radius,
                    center.1
                        + (std::f32::consts::TAU * index as f32 / segments as f32).sin() * radius,
                );
                let b = (
                    center.0
                        + (std::f32::consts::TAU * next as f32 / segments as f32).cos() * radius,
                    center.1
                        + (std::f32::consts::TAU * next as f32 / segments as f32).sin() * radius,
                );
                self.line(a, b, 2.0, depth, Color::BLACK);
            }
        }
    }

    fn into_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

pub fn setup_renderer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Geometry::default().into_mesh());
    let entity = commands
        .spawn((
            Mesh2d(mesh.clone()),
            MeshMaterial2d(materials.add(ColorMaterial::default())),
            Visibility::Hidden,
        ))
        .id();
    commands.insert_resource(WorldVisuals { entity, mesh });
}

pub fn render_game<M: GameMap, P: PlayerActions, N: MonsterBehavior>(
    game: &Game<M, P, N>,
    mouse: Option<(f32, f32)>,
    meshes: &mut Assets<Mesh>,
    visuals: &WorldVisuals,
) {
    let mut geometry = Geometry::default();
    let hovered = mouse.and_then(|mouse| game.hovered_tile(mouse));
    let preview_path = mouse.map_or_else(Vec::new, |mouse| game.preview_path(mouse));

    for depth in 0..(MAP * 2 - 1) {
        for y in 0..MAP {
            if y > depth {
                continue;
            }
            let x = depth - y;
            if x >= MAP {
                continue;
            }

            let (sx, sy) = to_screen(x, y, game.camera());
            let z = depth as f32 * 0.01;

            if game.map().tile((x, y)) == Some(Tile::Wall) {
                draw_wall(&mut geometry, sx, sy, z);
            } else {
                draw_floor(&mut geometry, sx, sy, z, hovered == Some((x, y)));

                if game.gold().contains(&(x, y)) {
                    geometry.circle(
                        (sx, sy - 24.0),
                        6.0,
                        z + 0.002,
                        Color::srgb(1.0, 0.84, 0.0),
                        true,
                    );
                }
            }

            if preview_path.contains(&(x, y)) {
                geometry.circle(
                    (sx, sy - 24.0),
                    4.0,
                    z + 0.003,
                    Color::srgb(0.9, 0.67, 0.0),
                    true,
                );
            }

            if game.player().position() == (x, y) {
                draw_character(&mut geometry, sx, sy, z + 0.004, false);
            }

            for monster in game.monsters() {
                if monster.position() == (x, y) {
                    draw_character(&mut geometry, sx, sy, z + 0.004, true);
                }
            }
        }
    }

    *meshes
        .get_mut(&visuals.mesh)
        .expect("game mesh asset must remain available") = geometry.into_mesh();
}

fn draw_floor(geometry: &mut Geometry, sx: f32, sy: f32, depth: f32, highlighted: bool) {
    let corners = [
        (sx, sy - 40.0),
        (sx + 32.0, sy - 24.0),
        (sx, sy - 8.0),
        (sx - 32.0, sy - 24.0),
    ];

    if highlighted {
        geometry.polygon(&corners, depth + 0.001, Color::srgb(0.65, 0.85, 1.0));
    }

    for edge in 0..corners.len() {
        geometry.line(
            corners[edge],
            corners[(edge + 1) % corners.len()],
            1.0,
            depth + 0.002,
            Color::srgb(0.65, 0.65, 0.65),
        );
    }
}

fn draw_wall(geometry: &mut Geometry, sx: f32, sy: f32, depth: f32) {
    let top = [
        (sx, sy - 72.0),
        (sx + 32.0, sy - 56.0),
        (sx, sy - 40.0),
        (sx - 32.0, sy - 56.0),
    ];
    let base = depth + 0.001;

    geometry.polygon(
        &[top[0], top[1], top[2], top[3]],
        base,
        Color::srgb(0.8, 0.8, 0.8),
    );
    geometry.polygon(
        &[top[1], (sx + 32.0, sy - 24.0), (sx, sy - 8.0), top[2]],
        base,
        Color::srgb(0.5, 0.5, 0.5),
    );
    geometry.polygon(
        &[top[3], top[2], (sx, sy - 8.0), (sx - 32.0, sy - 24.0)],
        base,
        Color::srgb(0.6, 0.6, 0.6),
    );

    for (a, b) in [
        (top[0], top[1]),
        (top[1], top[2]),
        (top[2], top[3]),
        (top[3], top[0]),
        (top[1], (sx + 32.0, sy - 24.0)),
        (top[2], (sx, sy - 8.0)),
        (top[3], (sx - 32.0, sy - 24.0)),
    ] {
        geometry.line(a, b, 1.0, base + 0.001, Color::BLACK);
    }
}

fn draw_character(geometry: &mut Geometry, sx: f32, sy: f32, depth: f32, enemy: bool) {
    let sy = sy - 24.0;
    geometry.ellipse(
        (sx, sy + 3.0),
        (10.0, 5.0),
        depth,
        Color::srgba(0.0, 0.0, 0.0, 0.2),
    );

    if enemy {
        geometry.line(
            (sx - 5.0, sy - 32.0),
            (sx, sy - 30.0),
            2.0,
            depth + 0.001,
            Color::BLACK,
        );
        geometry.line(
            (sx + 5.0, sy - 32.0),
            (sx, sy - 30.0),
            2.0,
            depth + 0.001,
            Color::BLACK,
        );
    } else {
        geometry.circle((sx, sy - 32.0), 7.0, depth + 0.001, Color::BLACK, false);
    }

    for [x1, y1, x2, y2] in [
        [0.0, -25.0, 0.0, -8.0],
        [0.0, -20.0, -8.0, -15.0],
        [0.0, -20.0, 8.0, -15.0],
        [0.0, -8.0, -6.0, 0.0],
        [0.0, -8.0, 6.0, 0.0],
    ] {
        geometry.line(
            (sx + x1, sy + y1),
            (sx + x2, sy + y2),
            2.0,
            depth + 0.001,
            Color::BLACK,
        );
    }
}

impl Geometry {
    fn ellipse(&mut self, center: (f32, f32), half_size: (f32, f32), depth: f32, color: Color) {
        let points: Vec<_> = (0..16)
            .map(|step| {
                let angle = std::f32::consts::TAU * step as f32 / 16.0;
                (
                    center.0 + angle.cos() * half_size.0,
                    center.1 + angle.sin() * half_size.1,
                )
            })
            .collect();
        self.polygon(&points, depth, color);
    }
}
