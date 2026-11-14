// native_hooks::Game::native_screen_mouse_button_pressed_pre (a hook point), HEAD.
// Mouse down on the world map.
fn megamap_screen_press(game: &crate::game::Game<'_>, screen: &horizon_client::mod_api::NativeScreenView<'_>, button: i32, canceled: &mut bool) {
    use crate::game::native_megamap as m;
    if !*canceled && game.native_on(m::ID) && m::screen_press(game, screen, button) {
        *canceled = true;
    }
}
