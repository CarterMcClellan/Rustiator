use rand::Rng;
use shakmaty::{Move, MoveList, Position};
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use log::{error, info};

use crate::chess_game::ChessGame;
use crate::websocket::Notification;

pub trait ChooseMove {
    fn choose_move(&self, fen: &str, legal_moves: &MoveList) -> Option<Move>;
}

pub struct RandomEngine {}

impl RandomEngine {
    pub fn new() -> Self {
        RandomEngine {}
    }
}

impl ChooseMove for RandomEngine {
    fn choose_move(&self, _chess_game: &str, legal_moves: &MoveList) -> Option<Move> {
        if legal_moves.is_empty() {
            None
        } else {
            thread::sleep(Duration::from_millis(250)); // Delay for 250 ms
            let mut rng = rand::thread_rng();
            let random_index = rng.gen_range(0..legal_moves.len());
            legal_moves.get(random_index).cloned()
        }
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
            let (legal_moves, fen) = (game.get_legal_moves(), game.fen());

            if let Some(m) = engine.choose_move(&fen, &legal_moves) {
                game.make_move(&m);
                send_notification(&sender_channel, game.fen());
            } else {
                info!("Game over, other engine wins or stalemate");
                return;
            }

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
