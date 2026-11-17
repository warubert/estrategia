use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui::{InteractionDisabled, Pressed},
    ui_widgets::Button,
};

#[derive(Component)]
pub struct WindowPanel;

#[derive(Component)]
pub struct WindowButton;

#[derive(Component)]
pub struct WindowButtonFocus;

pub struct WindowParts {
    pub panel: Entity,
    pub content: Entity,
}

pub fn spawn_window(
    commands: &mut Commands,
    font: &Handle<Font>,
    title: &str,
    mut layout: Node,
    visibility: Visibility,
) -> WindowParts {
    layout.padding = UiRect::all(px(14));
    layout.flex_direction = FlexDirection::Column;
    layout.align_items = AlignItems::Stretch;
    layout.row_gap = px(10);

    let panel = commands
        .spawn((
            WindowPanel,
            Hovered::default(),
            layout,
            BackgroundColor(Color::srgba(0.08, 0.08, 0.08, 0.92)),
            BorderColor::all(Color::srgb(0.55, 0.55, 0.55)),
            visibility,
        ))
        .id();
    let title = commands
        .spawn((
            Text::new(title),
            TextFont::from_font_size(FontSize::Px(22.0)).with_font(font.clone()),
            TextColor(Color::WHITE),
        ))
        .id();
    let content = commands
        .spawn((
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
            WindowContent,
        ))
        .id();
    commands.entity(panel).add_children(&[title, content]);

    WindowParts { panel, content }
}

#[derive(Component)]
pub struct WindowContent;

pub fn spawn_button(
    commands: &mut Commands,
    parent: Entity,
    font: &Handle<Font>,
    label: &str,
    enabled: bool,
) -> Entity {
    let button = commands
        .spawn((
            Button,
            Hovered::default(),
            WindowButton,
            Node {
                width: percent(100),
                min_height: px(40),
                padding: UiRect::all(px(8)),
                border: UiRect::all(px(1)),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.16, 0.16, 0.16)),
            BorderColor::all(Color::srgb(0.38, 0.38, 0.38)),
            children![(
                Text::new(label),
                TextFont::from_font_size(FontSize::Px(20.0)).with_font(font.clone()),
                TextColor(Color::WHITE),
            )],
        ))
        .id();
    commands.entity(parent).add_child(button);
    if !enabled {
        commands.entity(button).insert(InteractionDisabled);
    }
    button
}

pub fn update_button_appearance(
    mut buttons: Query<
        (
            &Hovered,
            Has<Pressed>,
            Has<InteractionDisabled>,
            Has<WindowButtonFocus>,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<WindowButton>,
    >,
) {
    for (hovered, pressed, disabled, focused, mut background, mut border) in &mut buttons {
        let (fill, outline) = if disabled {
            (Color::srgb(0.12, 0.12, 0.12), Color::srgb(0.25, 0.25, 0.25))
        } else if pressed {
            (Color::srgb(0.22, 0.34, 0.22), Color::srgb(0.45, 0.8, 0.45))
        } else if focused {
            (Color::srgb(0.26, 0.25, 0.12), Color::srgb(1.0, 0.84, 0.0))
        } else if hovered.get() {
            (Color::srgb(0.24, 0.24, 0.24), Color::srgb(0.75, 0.75, 0.75))
        } else {
            (Color::srgb(0.16, 0.16, 0.16), Color::srgb(0.38, 0.38, 0.38))
        };
        background.0 = fill;
        border.set_all(outline);
    }
}
