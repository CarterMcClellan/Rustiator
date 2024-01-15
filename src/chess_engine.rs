use rand::Rng;
use shakmaty::{Move, MoveList};
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use anyhow::Result as AnyResult;

use log::{error, info};

use crate::chess_game::ChessGame;
use crate::websocket::Notification;

pub trait ChooseMove: Send + Sync {
    /// Choose move has the assumption that the bot can play a move. Handing it a game with no legal moves
    /// is invalid. The user should check if the game is over before calling this
    fn choose_move(&self, chess_game: &ChessGame, legal_moves: &MoveList) -> AnyResult<Move>;
}

/// Allows us to distinguish between a saved bot and an active bot
/// In addition Clone is not allowed for dyn trait objects and this helps us get around that
pub trait ToChooseMove {
    fn to_choose_move(&self) -> Box<dyn ChooseMove>;
}

impl<C: 'static + ChooseMove + Clone> ToChooseMove for C {
    fn to_choose_move(&self) -> Box<dyn ChooseMove> {
        Box::new(self.clone())
    }
}

impl ChooseMove for Box<dyn ChooseMove> {
    fn choose_move(&self, chess_game: &ChessGame, legal_moves: &MoveList) -> AnyResult<Move> {
        // need to make sure we call the choose_move of the dyn object not the choose_move of the box
        (**self).choose_move(chess_game, legal_moves)
    }
}

#[derive(Clone)]
pub struct RandomEngine {}

impl RandomEngine {
    pub fn new() -> Self {
        RandomEngine {}
    }
}

impl ChooseMove for RandomEngine {
    fn choose_move(&self, _chess_game: &ChessGame, legal_moves: &MoveList) -> AnyResult<Move> {
        thread::sleep(Duration::from_millis(250)); // Delay for 250 ms
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..legal_moves.len());
        legal_moves.get(random_index).cloned().ok_or_else(|| {
            anyhow::anyhow!("Random Bot could not make a move since there are no legal moves")
        })
    }
}

pub fn engine_vs_engine<T: ChooseMove>(
    game: Arc<RwLock<ChessGame>>,
    engine1: Arc<T>,
    engine2: Arc<T>,
    sender_channel: Sender<Notification>,
) {
    info!("Engine vs Engine Started...");

    loop {
        let mut game = game.write().unwrap(); // Lock the game for the current scope

        // Check for game end conditions
        if game.game_over() {
            break;
        }

        // Alternate turns between Engine 1 and Engine 2
        for engine in [&engine1, &engine2].iter() {
            let legal_moves = game.get_legal_moves();

            let m = match engine.choose_move(&game, &legal_moves) {
                Ok(m) => m,
                Err(e) => {
                    error!("Bot threw error while choosing move: {e}");
                    return;
                }
            };

            game.make_move(&m);
            send_notification(&sender_channel, game.fen());

            if game.game_over() {
                return;
            }
        }
    }
}

fn send_notification(sender: &Sender<Notification>, fen: String) {
    match sender.send(Notification(fen)) {
        Ok(_) => {}
        Err(e) => error!("Error sending notification: {}", e),
    }
}
