use bevy::prelude::*;

mod entities;
mod game;
mod mapas;
mod monstros;
mod personagens;
mod renderer;

use bevy::{sprite::Anchor, window::PrimaryWindow};
use game::Game;
use personagens::PlayerActions;
use renderer::WorldVisuals;
use std::collections::{HashMap, HashSet};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppState {
    Menu,
    Playing,
    GameOver,
}

#[derive(Resource)]
struct GameSession {
    game: Game,
    state: AppState,
}

#[derive(Resource)]
struct UiEntities {
    menu: Entity,
    hud: Entity,
    game_over_title: Entity,
    game_over_score: Entity,
    restart_prompt: Entity,
    overlay: Entity,
}

#[derive(Resource, Default)]
struct FloatingTextEntities(HashMap<u64, Entity>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Crablo".to_owned(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::WHITE))
        .init_resource::<FloatingTextEntities>()
        .add_systems(Startup, (setup_game, renderer::setup_renderer))
        .add_systems(
            Update,
            (handle_input, sync_floating_texts, update_ui, render_world).chain(),
        )
        .run();
}

fn setup_game(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 100.0)));
    commands.insert_resource(GameSession {
        game: Game::new(),
        state: AppState::Menu,
    });

    let menu = spawn_text(
        &mut commands,
        "Menu - pressione Enter para começar",
        36.0,
        Color::BLACK,
        Anchor::CENTER,
        Vec3::new(0.0, 160.0, 3.0),
        Visibility::Visible,
    );
    let hud = spawn_text(
        &mut commands,
        "",
        24.0,
        Color::BLACK,
        Anchor::BOTTOM_LEFT,
        Vec3::new(-620.0, -340.0, 3.0),
        Visibility::Hidden,
    );
    let game_over_title = spawn_text(
        &mut commands,
        "",
        60.0,
        Color::srgb(0.8, 0.0, 0.0),
        Anchor::CENTER,
        Vec3::new(0.0, 30.0, 3.0),
        Visibility::Hidden,
    );
    let game_over_score = spawn_text(
        &mut commands,
        "",
        30.0,
        Color::BLACK,
        Anchor::CENTER,
        Vec3::new(0.0, -30.0, 3.0),
        Visibility::Hidden,
    );
    let restart_prompt = spawn_text(
        &mut commands,
        "Enter para voltar ao menu",
        20.0,
        Color::srgb(0.5, 0.5, 0.5),
        Anchor::CENTER,
        Vec3::new(0.0, -75.0, 3.0),
        Visibility::Hidden,
    );

    let overlay = commands
        .spawn((
            Sprite::from_color(
                Color::srgba(1.0, 1.0, 1.0, 0.7),
                Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
            ),
            Transform::from_xyz(0.0, 0.0, 1.0),
            Visibility::Hidden,
        ))
        .id();

    commands.insert_resource(UiEntities {
        menu,
        hud,
        game_over_title,
        game_over_score,
        restart_prompt,
        overlay,
    });
}

fn spawn_text(
    commands: &mut Commands,
    text: &str,
    size: f32,
    color: Color,
    anchor: Anchor,
    position: Vec3,
    visibility: Visibility,
) -> Entity {
    commands
        .spawn((
            Text2d::new(text),
            TextFont::from_font_size(FontSize::Px(size)),
            TextColor(color),
            anchor,
            Transform::from_translation(position),
            visibility,
        ))
        .id()
}

fn handle_input(
    mut session: ResMut<GameSession>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        match session.state {
            AppState::Menu => {
                session.game = Game::new();
                session.state = AppState::Playing;
            }
            AppState::GameOver => session.state = AppState::Menu,
            AppState::Playing => {}
        }
    }

    if session.state != AppState::Playing {
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Left)
        && let Some(cursor) = windows
            .single()
            .expect("exactly one primary window must exist")
            .cursor_position()
    {
        session.game.click((cursor.x, cursor.y));
    }

    if session.game.update(time.delta_secs()) {
        session.state = AppState::GameOver;
    }
}

fn update_ui(
    session: Res<GameSession>,
    ui: Res<UiEntities>,
    mut texts: Query<&mut Text2d>,
    mut colors: Query<&mut TextColor>,
    mut visibility: Query<&mut Visibility>,
) {
    let (menu, hud, title, score, prompt, overlay) = match session.state {
        AppState::Menu => (true, false, false, false, false, false),
        AppState::Playing => (false, true, false, false, false, false),
        AppState::GameOver => (false, false, true, true, true, true),
    };

    set_text(
        &mut texts,
        ui.hud,
        format!(
            "HP: {}\nScore: {}\n{}",
            session.game.player().health(),
            session.game.player().score(),
            session.game.turn_text()
        ),
    );
    set_text(
        &mut texts,
        ui.game_over_title,
        if session.game.player().health() > 0 {
            "VITÓRIA"
        } else {
            "GAME OVER"
        }
        .to_owned(),
    );
    set_text(
        &mut texts,
        ui.game_over_score,
        format!("Pontuação: {}", session.game.player().score()),
    );
    if let Ok(mut color) = colors.get_mut(ui.game_over_title) {
        color.0 = if session.game.player().health() > 0 {
            Color::srgb(1.0, 0.84, 0.0)
        } else {
            Color::srgb(0.8, 0.0, 0.0)
        };
    }

    set_visibility(&mut visibility, ui.menu, menu);
    set_visibility(&mut visibility, ui.hud, hud);
    set_visibility(&mut visibility, ui.game_over_title, title);
    set_visibility(&mut visibility, ui.game_over_score, score);
    set_visibility(&mut visibility, ui.restart_prompt, prompt);
    set_visibility(&mut visibility, ui.overlay, overlay);
}

fn sync_floating_texts(
    mut commands: Commands,
    session: Res<GameSession>,
    mut entities: ResMut<FloatingTextEntities>,
    mut text_query: Query<(&mut Text2d, &mut Transform)>,
) {
    let active_texts = if session.state == AppState::Menu {
        &[][..]
    } else {
        session.game.floating_texts()
    };
    let active_ids: HashSet<_> = active_texts.iter().map(|text| text.id).collect();
    entities.0.retain(|id, entity| {
        if active_ids.contains(id) {
            true
        } else {
            commands.entity(*entity).despawn();
            false
        }
    });

    for floating_text in active_texts {
        let label = if floating_text.dmg < 0 {
            format!("+{}", floating_text.dmg.unsigned_abs())
        } else {
            format!("-{}", floating_text.dmg)
        };
        let color = if floating_text.dmg < 0 {
            Color::srgb(0.0, 0.6, 0.0)
        } else {
            Color::srgb(0.8, 0.0, 0.0)
        };
        let position = Vec3::new(
            floating_text.x - WINDOW_WIDTH as f32 / 2.0,
            WINDOW_HEIGHT as f32 / 2.0 - floating_text.y,
            0.5,
        );

        if let Some(entity) = entities.0.get(&floating_text.id) {
            let (mut text, mut transform) = text_query
                .get_mut(*entity)
                .expect("floating text entity must remain available");
            if text.0 != label {
                text.0 = label;
            }
            transform.translation = position;
        } else {
            let entity = commands
                .spawn((
                    Text2d::new(label),
                    TextFont::from_font_size(FontSize::Px(20.0)),
                    TextColor(color),
                    Anchor::BOTTOM_LEFT,
                    Transform::from_translation(position),
                ))
                .id();
            entities.0.insert(floating_text.id, entity);
        }
    }
}

fn set_text(texts: &mut Query<&mut Text2d>, entity: Entity, value: String) {
    let mut text = texts
        .get_mut(entity)
        .expect("UI text entity must remain available");
    if text.0 != value {
        text.0 = value;
    }
}

fn set_visibility(visibility: &mut Query<&mut Visibility>, entity: Entity, visible: bool) {
    *visibility
        .get_mut(entity)
        .expect("UI entity must remain available") = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

fn render_world(
    session: Res<GameSession>,
    windows: Query<&Window, With<PrimaryWindow>>,
    visuals: Res<WorldVisuals>,
    mut visibility: Query<&mut Visibility>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    *visibility
        .get_mut(visuals.entity)
        .expect("game mesh entity must remain available") = if session.state == AppState::Menu {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };

    let cursor = windows
        .single()
        .expect("exactly one primary window must exist")
        .cursor_position()
        .map(|position| (position.x, position.y));
    renderer::render_game(&session.game, cursor, &mut meshes, &visuals);
}
