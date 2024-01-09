use log::{info, error};
use shakmaty::Move;
use anyhow::Result;

use crate::{chess_engine::ChooseMove, chess_game::ChessGame};

pub struct PlayerGame {
    bot: Box<dyn ChooseMove + Send + Sync>,
    pub game: ChessGame,
}

impl PlayerGame {
    pub fn new<C: 'static + ChooseMove + Send + Sync>(bot: C) -> Self {
        Self {
            bot: Box::new(bot),
            game: ChessGame::new(),
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

        // FIXME: remove unwrap. What does `None` mean for a choose move? it ran out of time?
        let bot_move = match self
            .bot
            .choose_move(&self.game.fen(), &legal_moves) {
                Some(m) => m,
                None => {
                    // not really sure what we are supposed to do here
                    // this is not a mistake by the player its a mistake by the bot
                    error!("Despite the game not being over, 
                        the bot returned None for a move. Game FEN {}.
                        Defaulting to a random move", self.game.fen());

                    // as mentioned above, the game not being over should
                    // guarantee that there are legal moves 
                    if legal_moves.is_empty() {
                        let msg = format!(
                            "Despite the game not being over, the bot returned None for a move. 
                            Game FEN {}. There are no legal moves",
                            self.game.fen()
                        );
                        error!("{}", msg);
                        return Err(anyhow::anyhow!(msg));
                    }

                    legal_moves[0].clone()
                }
            };

        self.game.make_move(&bot_move);

        Ok(Some(bot_move))
    }

    pub fn fen(&self) -> String {
        self.game.fen()
    }
}
