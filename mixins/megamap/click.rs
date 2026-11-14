// native_hooks::Game::native_settings_click (a hook point), HEAD.
// A click on the mod's settings page: change the setting and keep it.
fn megamap_click(mod_id: &str, button: usize) {
    use crate::game::native_megamap as m;
    m::settings_click(mod_id, button);
}
