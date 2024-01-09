use shakmaty::Move;

use crate::{chess_engine::ChooseMove, chess_game::ChessGame};

pub struct PlayerGame {
    bot: Box<dyn ChooseMove + Send + Sync>,
    game: ChessGame,
}

impl PlayerGame {
    pub fn new<C: 'static + ChooseMove + Send + Sync>(bot: C) -> Self {
        Self {
            bot: Box::new(bot),
            game: ChessGame::new(),
        }
    }

    /// Takes in player move and returns bot move
    pub fn play_move(&mut self, player_move: Move) -> Move {
        self.game.make_move(player_move);
        // FIXME: remove unwrap. What does `None` mean for a choose move? it ran out of time?
        self.bot
            .choose_move(&self.game.fen(), &self.game.get_legal_moves())
            .unwrap()
    }
}
