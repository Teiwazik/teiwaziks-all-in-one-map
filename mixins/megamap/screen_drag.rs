// native_hooks::Game::native_screen_mouse_dragged_pre (a hook point), HEAD.
// Dragging the world map.
fn megamap_screen_drag(game: &crate::game::Game<'_>, screen: &horizon_client::mod_api::NativeScreenView<'_>, delta: [f32; 2], canceled: &mut bool) {
    use crate::game::native_megamap as m;
    if !*canceled && game.native_on(m::ID) && m::screen_drag(game, screen, delta) {
        *canceled = true;
    }
}
