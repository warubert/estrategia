use bevy::prelude::*;

mod entities;
mod game;
mod mapas;
mod monstros;
mod personagens;
mod renderer;
mod ui;

use bevy::{
    picking::hover::Hovered, sprite::Anchor, ui::InteractionDisabled, window::PrimaryWindow,
};
use game::Game;
use personagens::PlayerActions;
use renderer::WorldVisuals;
use std::collections::{HashMap, HashSet};
use ui::action_menu::{
    ActionMenuButton, ActionMenuState, ActionMenuWindow, ActionMode, ActionOption,
};
use ui::window::{self, WindowButtonFocus};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const GAME_FONT_PATH: &str = "fonts/Lato-Regular.ttf";

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
        .init_resource::<ActionMenuState>()
        .add_systems(Startup, (setup_game, renderer::setup_renderer))
        .add_systems(
            Update,
            (
                handle_input,
                sync_floating_texts,
                update_ui,
                update_action_menu,
                window::update_button_appearance,
                render_world,
            )
                .chain(),
        )
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(GAME_FONT_PATH);
    commands.spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 100.0)));
    ui::action_menu::spawn_action_menu(&mut commands, &font);
    commands.insert_resource(GameSession {
        game: Game::new(),
        state: AppState::Menu,
    });

    let menu = spawn_text(
        &mut commands,
        &font,
        "Menu - pressione Enter para começar",
        36.0,
        Color::BLACK,
        Anchor::CENTER,
        Vec3::new(0.0, 160.0, 3.0),
        Visibility::Visible,
    );
    let hud = spawn_text(
        &mut commands,
        &font,
        "",
        24.0,
        Color::BLACK,
        Anchor::BOTTOM_LEFT,
        Vec3::new(-620.0, -340.0, 3.0),
        Visibility::Hidden,
    );
    let game_over_title = spawn_text(
        &mut commands,
        &font,
        "",
        60.0,
        Color::srgb(0.8, 0.0, 0.0),
        Anchor::CENTER,
        Vec3::new(0.0, 30.0, 3.0),
        Visibility::Hidden,
    );
    let game_over_score = spawn_text(
        &mut commands,
        &font,
        "",
        30.0,
        Color::BLACK,
        Anchor::CENTER,
        Vec3::new(0.0, -30.0, 3.0),
        Visibility::Hidden,
    );
    let restart_prompt = spawn_text(
        &mut commands,
        &font,
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
    font: &Handle<Font>,
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
            TextFont::from_font_size(FontSize::Px(size)).with_font(font.clone()),
            TextColor(color),
            anchor,
            Transform::from_translation(position),
            visibility,
        ))
        .id()
}

fn handle_input(
    mut session: ResMut<GameSession>,
    mut action_menu: ResMut<ActionMenuState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    action_buttons: Query<(&ActionMenuButton, &Hovered, Has<InteractionDisabled>)>,
    action_window_hover: Query<&Hovered, With<ActionMenuWindow>>,
    time: Res<Time>,
) {
    if session.state == AppState::Playing {
        if keyboard.just_pressed(KeyCode::Escape) {
            action_menu.back();
        } else if session.game.is_player_action_turn() && action_menu.mode() == ActionMode::Menu {
            if keyboard.just_pressed(KeyCode::ArrowUp) {
                action_menu.navigate(-1, session.game.can_attack());
            }
            if keyboard.just_pressed(KeyCode::ArrowDown) {
                action_menu.navigate(1, session.game.can_attack());
            }
            if keyboard.just_pressed(KeyCode::Enter) {
                let action = action_menu.activate_selected(session.game.can_attack());
                apply_menu_action(&mut session.game, &mut action_menu, action);
            }
        }
    } else if keyboard.just_pressed(KeyCode::Enter) {
        match session.state {
            AppState::Menu => {
                session.game = Game::new();
                action_menu.reset();
                session.state = AppState::Playing;
            }
            AppState::GameOver => session.state = AppState::Menu,
            AppState::Playing => {}
        }
    }

    if session.state != AppState::Playing {
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(option) = ui::action_menu::hovered_action(&action_buttons) {
            let action = action_menu.activate(option, session.game.can_attack());
            apply_menu_action(&mut session.game, &mut action_menu, action);
        } else if !action_window_hover.iter().any(|hovered| hovered.get())
            && session.game.is_player_action_turn()
            && let Some(cursor) = windows
                .single()
                .expect("exactly one primary window must exist")
                .cursor_position()
            && let Some(target) = session.game.hovered_tile((cursor.x, cursor.y))
        {
            let action_consumed = match action_menu.mode() {
                ActionMode::Menu => false,
                ActionMode::MoveTarget => session.game.move_player_to(target),
                ActionMode::AttackTarget => session.game.attack_monster_at(target),
            };
            if action_consumed {
                action_menu.finish_action();
            }
        }
    }

    if session.game.update(time.delta_secs()) {
        session.state = AppState::GameOver;
    }
}

fn apply_menu_action(
    game: &mut Game,
    action_menu: &mut ActionMenuState,
    action: Option<ActionOption>,
) {
    if action == Some(ActionOption::EndTurn) {
        game.end_player_turn();
        action_menu.finish_action();
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
            "HP: {}\nPontuação: {}\n{}",
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

fn update_action_menu(
    session: Res<GameSession>,
    action_menu: Res<ActionMenuState>,
    mut windows: Query<&mut Visibility, With<ActionMenuWindow>>,
    mut buttons: Query<(
        Entity,
        &ActionMenuButton,
        Has<InteractionDisabled>,
        Has<WindowButtonFocus>,
    )>,
    mut commands: Commands,
) {
    let visible = session.state == AppState::Playing && session.game.is_player_action_turn();
    for mut visibility in &mut windows {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (entity, button, disabled, focused) in &mut buttons {
        let should_disable = button.0 == ActionOption::Attack && !session.game.can_attack();
        if should_disable != disabled {
            if should_disable {
                commands.entity(entity).insert(InteractionDisabled);
            } else {
                commands.entity(entity).remove::<InteractionDisabled>();
            }
        }

        let should_focus = action_menu.selected() == button.0;
        if should_focus != focused {
            if should_focus {
                commands.entity(entity).insert(WindowButtonFocus);
            } else {
                commands.entity(entity).remove::<WindowButtonFocus>();
            }
        }
    }
}

fn sync_floating_texts(
    mut commands: Commands,
    session: Res<GameSession>,
    asset_server: Res<AssetServer>,
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
                    TextFont::from_font_size(FontSize::Px(20.0))
                        .with_font(asset_server.load(GAME_FONT_PATH)),
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
    action_menu: Res<ActionMenuState>,
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
    let preview_enabled =
        action_menu.mode() == ActionMode::MoveTarget || session.game.is_player_moving();
    renderer::render_game(
        &session.game,
        cursor,
        preview_enabled,
        &mut meshes,
        &visuals,
    );
}
