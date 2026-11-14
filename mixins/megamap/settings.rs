// native_hooks::Game::native_settings (a hook point), HEAD.
// The mod's settings page.
fn megamap_settings(mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::native_megamap as m;
    m::settings(mod_id, buttons);
}
