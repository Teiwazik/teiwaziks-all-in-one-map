// native_hooks::Game::native_mods (a hook point), HEAD.
// The mod's row in Options > Mods, with its keys. Listed when turned off too, so it can be turned on.
fn megamap_list(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::native_megamap as m;
    m::mods(mods);
}
