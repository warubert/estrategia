use super::window::{self, WindowParts};
use bevy::{picking::hover::Hovered, prelude::*, ui::InteractionDisabled};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionOption {
    Move,
    Attack,
    EndTurn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionMode {
    Menu,
    MoveTarget,
    AttackTarget,
}

#[derive(Resource)]
pub struct ActionMenuState {
    selected: ActionOption,
    mode: ActionMode,
}

impl Default for ActionMenuState {
    fn default() -> Self {
        Self {
            selected: ActionOption::Move,
            mode: ActionMode::Menu,
        }
    }
}

impl ActionMenuState {
    pub fn selected(&self) -> ActionOption {
        self.selected
    }

    pub fn mode(&self) -> ActionMode {
        self.mode
    }

    pub fn navigate(&mut self, direction: isize, attack_enabled: bool) {
        if self.mode != ActionMode::Menu || direction == 0 {
            return;
        }

        let options = [
            ActionOption::Move,
            ActionOption::Attack,
            ActionOption::EndTurn,
        ];
        let current = options
            .iter()
            .position(|option| *option == self.selected)
            .expect("selected action must be present in the action menu");

        for offset in 1..=options.len() {
            let index = (current as isize + direction.signum() * offset as isize)
                .rem_euclid(options.len() as isize) as usize;
            let option = options[index];
            if option != ActionOption::Attack || attack_enabled {
                self.selected = option;
                return;
            }
        }
    }

    pub fn activate_selected(&mut self, attack_enabled: bool) -> Option<ActionOption> {
        self.activate(self.selected, attack_enabled)
    }

    pub fn activate(&mut self, option: ActionOption, attack_enabled: bool) -> Option<ActionOption> {
        if self.mode != ActionMode::Menu || (option == ActionOption::Attack && !attack_enabled) {
            return None;
        }

        self.selected = option;
        self.mode = match option {
            ActionOption::Move => ActionMode::MoveTarget,
            ActionOption::Attack => ActionMode::AttackTarget,
            ActionOption::EndTurn => ActionMode::Menu,
        };
        Some(option)
    }

    pub fn back(&mut self) -> bool {
        if self.mode == ActionMode::Menu {
            return false;
        }
        self.mode = ActionMode::Menu;
        true
    }

    pub fn finish_action(&mut self) {
        self.mode = ActionMode::Menu;
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Component)]
pub struct ActionMenuWindow;

#[derive(Component)]
pub struct ActionMenuButton(pub ActionOption);

pub fn spawn_action_menu(commands: &mut Commands, font: &Handle<Font>) {
    let WindowParts { panel, content } = window::spawn_window(
        commands,
        font,
        "Ações do jogador",
        Node {
            position_type: PositionType::Absolute,
            right: px(16),
            bottom: px(16),
            width: px(320),
            ..default()
        },
        Visibility::Hidden,
    );
    commands.entity(panel).insert(ActionMenuWindow);

    let move_button = window::spawn_button(commands, content, font, "Mover", true);
    commands
        .entity(move_button)
        .insert(ActionMenuButton(ActionOption::Move));

    let attack_button = window::spawn_button(commands, content, font, "Atacar", false);
    commands
        .entity(attack_button)
        .insert(ActionMenuButton(ActionOption::Attack));

    let end_turn_button = window::spawn_button(commands, content, font, "Encerrar turno", true);
    commands
        .entity(end_turn_button)
        .insert(ActionMenuButton(ActionOption::EndTurn));
}

pub fn hovered_action(
    buttons: &Query<(&ActionMenuButton, &Hovered, Has<InteractionDisabled>)>,
) -> Option<ActionOption> {
    buttons
        .iter()
        .find_map(|(button, hovered, disabled)| (hovered.get() && !disabled).then_some(button.0))
}

#[cfg(test)]
#[path = "../../tests/unit/action_menu.rs"]
mod tests;
