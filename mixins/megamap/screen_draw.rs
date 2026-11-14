// native_hooks::Game::native_screen_render_post (a hook point), HEAD.
// Over megamap's own mod screen: the world map.
fn megamap_screen_draw(game: &crate::game::Game<'_>, batch: &mut horizon_client::mod_api::HudBatch, screen: &horizon_client::mod_api::NativeScreenView<'_>) {
    use crate::game::native_megamap as m;
    if game.native_on(m::ID) {
        m::screen_draw(game, batch, screen);
    }
}
