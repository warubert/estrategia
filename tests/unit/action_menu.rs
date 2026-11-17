use super::*;

#[test]
fn keyboard_navigation_skips_disabled_attack_and_wraps() {
    let mut menu = ActionMenuState::default();

    menu.navigate(1, false);
    assert_eq!(menu.selected(), ActionOption::EndTurn);
    menu.navigate(1, false);
    assert_eq!(menu.selected(), ActionOption::Move);

    menu.navigate(1, true);
    assert_eq!(menu.selected(), ActionOption::Attack);
}

#[test]
fn target_actions_can_be_cancelled_without_consuming_a_game_turn() {
    let mut menu = ActionMenuState::default();

    assert_eq!(menu.activate_selected(false), Some(ActionOption::Move));
    assert_eq!(menu.mode(), ActionMode::MoveTarget);
    assert!(menu.back());
    assert_eq!(menu.mode(), ActionMode::Menu);

    assert_eq!(
        menu.activate(ActionOption::Attack, true),
        Some(ActionOption::Attack)
    );
    assert_eq!(menu.mode(), ActionMode::AttackTarget);
    assert!(menu.back());
    assert_eq!(menu.mode(), ActionMode::Menu);
}

#[test]
fn disabled_attack_cannot_be_activated_and_end_turn_is_a_command() {
    let mut menu = ActionMenuState::default();

    assert_eq!(menu.activate(ActionOption::Attack, false), None);
    assert_eq!(menu.mode(), ActionMode::Menu);

    assert_eq!(
        menu.activate(ActionOption::EndTurn, false),
        Some(ActionOption::EndTurn)
    );
    assert_eq!(menu.mode(), ActionMode::Menu);
}
