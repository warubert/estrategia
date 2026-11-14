use crate::mapas::to_screen;
use macroquad::prelude::*;

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

    draw_triangle(v[0], v[1], v[2], colors[0]);
    draw_triangle(v[0], v[2], v[3], colors[0]);
    draw_triangle(v[1], v[4], v[5], colors[1]);
    draw_triangle(v[1], v[5], v[2], colors[1]);
    draw_triangle(v[3], v[2], v[5], colors[2]);
    draw_triangle(v[3], v[5], v[6], colors[2]);

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

//draw hero and monsters
pub fn draw_stickman(x: usize, y: usize, cam: (f32, f32), enemy: bool) {
    let (sx, sy) = to_screen(x, y, cam);
    let sy = sy - 24.;

    //shadow
    draw_ellipse(sx, sy + 3., 10., 5., 0., Color::new(0., 0., 0., 0.2));

    //head
    if enemy {
        draw_line(sx - 5., sy - 32., sx, sy - 30., 2., BLACK);
        draw_line(sx + 5., sy - 32., sx, sy - 30., 2., BLACK);
    } else {
        //player
        draw_circle_lines(sx, sy - 32., 7., 2., BLACK);
    }

    //body and limbs
    for l in [
        [0., -25., 0., -8.],
        [0., -20., -8., -15.],
        [0., -20., 8., -15.],
        [0., -8., -6., 0.],
        [0., -8., 6., 0.],
    ] {
        draw_line(sx + l[0], sy + l[1], sx + l[2], sy + l[3], 2., BLACK);
    }
}
