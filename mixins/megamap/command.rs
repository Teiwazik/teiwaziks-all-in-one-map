// native_hooks::Game::native_command (a hook point), HEAD.
// `/megamap ...`; everything else goes on to the game and other mods.
fn megamap_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    use crate::game::native_megamap as m;
    if !*ran && game.native_on(m::ID) && m::command(game, command) {
        *ran = true;
    }
}
