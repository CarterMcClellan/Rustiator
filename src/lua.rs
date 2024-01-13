use mlua::{Lua, StdLib};

use anyhow::{Context, Result as AnyResult};

use shakmaty::{uci::Uci, CastlingMode, MoveList};

use crate::{chess_engine::ChooseMove, chess_game::ChessGame};

/// Simplest implentation of a lua bot is stateless.
/// Each time a move is chosen this script is evaluated by a new lua vm to generate the bot
///
/// This means that any state in the bot will not be preserved on successive calls to choose_move
#[derive(Clone)]
pub struct StatelessLuaBot {
    pub script: String,
}

impl StatelessLuaBot {
    pub fn try_new(script: String) -> AnyResult<Self> {
        validate_script(&script)?;
        Ok(Self { script })
    }
}

impl ChooseMove for StatelessLuaBot {
    fn choose_move(
        &self,
        chess_game: &ChessGame,
        legal_moves: &MoveList,
    ) -> AnyResult<shakmaty::Move> {
        let vm = init_vm();
        let chunk = vm.load(&self.script);
        let bot = chunk
            .eval()
            .context("Error initializing bot while choosing move")?;

        let uci_legal_moves = legal_moves
            .iter()
            .map(|legal_move| legal_move.to_uci(CastlingMode::Standard).to_string())
            .collect::<Vec<_>>();

        let chosen_uci: Uci = invoke_bot(bot, &chess_game.fen(), &uci_legal_moves)
            .context("Bot made an error while choosing its move")?
            .parse()
            // TODO: confirm that this error message will include the string the bot returned
            .context("Error parsing bot response as uci")?;

        log::debug!("Bot trying to play move: {chosen_uci}");

        let chosen_move = chosen_uci
            .to_move(&chess_game.game)
            .context("Bot returned illegal move")?;

        Ok(chosen_move)
    }
}

fn invoke_bot(bot: mlua::Table<'_>, chess_game: &str, legal_moves: &[String]) -> AnyResult<String> {
    let choose_move: mlua::Function = bot.get("chooseMove")?;
    let chosen_move = choose_move.call((bot, chess_game, legal_moves))?;
    Ok(chosen_move)
}

fn init_vm() -> Lua {
    // the Anti-Eamonn policy
    let whitelisted_libs = StdLib::MATH | StdLib::TABLE | StdLib::STRING;
    // SAFETY: This function only errors if we use StdLib::DEBUG or StdLib::FFI
    Lua::new_with(whitelisted_libs, mlua::LuaOptions::default()).unwrap()
}

/// TODO:
/// will probably want to try to generate the bot and check that chooseMove exists
/// valide it is a function and try passing it a starting position as a simple dummy check
fn validate_script(_script: &str) -> AnyResult<()> {
    Ok(())
}
