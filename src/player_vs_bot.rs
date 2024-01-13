use anyhow::Result;
use shakmaty::Move;

use crate::{chess_engine::ChooseMove, chess_game::ChessGame};

pub struct PlayerGame {
    bot: Box<dyn ChooseMove>,
    pub game: ChessGame,
    pub bot_name: String,
}

impl PlayerGame {
    pub fn new<C: 'static + ChooseMove>(bot: C, name: impl ToString) -> Self {
        Self {
            bot: Box::new(bot),
            game: ChessGame::new(),
            bot_name: name.to_string(),
        }
    }

    /// Takes in player move and then playes the bot move that responds to this. Also returns the move
    pub fn play_move(&mut self, player_move: Move) -> Result<Option<Move>> {
        self.game.make_move(&player_move);

        let legal_moves = &self.game.get_legal_moves();

        // legal moves should be a stronger condition
        // than game over, there are scenario (like the 50 move rule)
        // where the game is over but there are still legal moves
        if self.game.game_over() {
            return Ok(None);
        }

        // Going to propagate error here from bot. For lua bots we have to assume that
        // bugs are common and we want to propogate them to the client so bot creators can debug
        let bot_move = self.bot.choose_move(&self.game, legal_moves)?;

        self.game.make_move(&bot_move);

        Ok(Some(bot_move))
    }

    pub fn fen(&self) -> String {
        self.game.fen()
    }
}
