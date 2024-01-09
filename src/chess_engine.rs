use rand::Rng;
use shakmaty::{Move, MoveList, Position};
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use log::{info, error};

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
    sender_channel: Sender<Notification>
) {
    info!("Engine vs Engine Started...");
    let mut moves_without_capture = 0;

    loop {
        let mut game = game.write().unwrap(); // Lock the game for the current scope

        // Check for game end conditions
        if game_ended(&game, moves_without_capture) {
            break;
        }

        // Alternate turns between Engine 1 and Engine 2
        for engine in [&engine1, &engine2].iter() {
            let (legal_moves, fen) = (game.get_legal_moves(), game.fen());

            if let Some(m) = engine.choose_move(&fen, &legal_moves) {
                update_moves_counter(&mut moves_without_capture, m.is_capture());
                game.make_move(&m);
                send_notification(&sender_channel, game.fen());
            } else {
                info!("Game over, other engine wins or stalemate");
                return;
            }

            if game_ended(&game, moves_without_capture) {
                return;
            }
        }
    }
}

fn game_ended(game: &ChessGame, moves_without_capture: usize) -> bool {
    if game.game.is_checkmate() || game.game.is_stalemate() || 
       game.game.is_insufficient_material() || game.game.outcome().is_some() ||
       moves_without_capture >= 50 
    {
        info!("Game over, endgame condition reached");
        true
    } else {
        false
    }
}

fn update_moves_counter(counter: &mut usize, is_capture: bool) {
    if is_capture {
        *counter = 0;
    } else {
        *counter += 1;
    }
}

fn send_notification(sender: &Sender<Notification>, fen: String) {
    match sender.send(Notification(fen)) {
        Ok(_) => {},
        Err(e) => error!("Error sending notification: {}", e),
    }
}