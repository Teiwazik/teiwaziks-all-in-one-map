// native_hooks::Game::native_hud (a hook point), HEAD.
// The HUD: the minimap in its corner, toasts.
fn megamap_hud(game: &mut crate::game::Game<'_>, batch: &mut horizon_client::mod_api::HudBatch, corners: &mut horizon_client::mod_api::Corners) {
    use crate::game::native_megamap as m;
    if game.native_on(m::ID) {
        m::hud(game, batch, corners);
    }
}
