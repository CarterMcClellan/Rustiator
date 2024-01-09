// 99% of the code in this file is just serialization/deserialization code
// the only interesting bit is the logic at the botton for actually maintaining
// the game state
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use shakmaty::{Chess, Move, MoveList, Position, Role, Square};

pub struct ChessGame {
    pub game: Chess,
}

// Serde calls this the definition of the remote type. It is just a copy of the
// remote data structure. The `remote` attribute gives the path to the actual
// type we intend to derive code for.
#[derive(Serialize, Deserialize, Clone)]
#[serde(remote = "Role")]
pub enum RoleDef {
    Pawn = 1,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
}

// Implementing Serialize for RoleDef.
impl Serialize for RoleDef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match *self {
            RoleDef::Pawn => 1,
            RoleDef::Knight => 2,
            RoleDef::Bishop => 3,
            RoleDef::Rook => 4,
            RoleDef::Queen => 5,
            RoleDef::King => 6,
        };
        serializer.serialize_u8(value)
    }
}

// Implementing Deserialize for RoleDef.
impl<'de> Deserialize<'de> for RoleDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            1 => Ok(RoleDef::Pawn),
            2 => Ok(RoleDef::Knight),
            3 => Ok(RoleDef::Bishop),
            4 => Ok(RoleDef::Rook),
            5 => Ok(RoleDef::Queen),
            6 => Ok(RoleDef::King),
            _ => Err(serde::de::Error::custom(format!(
                "unknown RoleDef value: {}",
                value
            ))),
        }
    }
}

// ugh: https://github.com/serde-rs/serde/issues/1301
mod opt_external_struct {
    use super::{Role, RoleDef};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<Role>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "RoleDef")] &'a Role);

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Role>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "RoleDef")] Role);

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Square")]
pub enum SquareDef {
    A1 = 0,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
}

pub fn movelist_to_vec(moves: MoveList) -> Vec<Move> {
    let mut vec = Vec::new();
    for m in moves {
        vec.push(m);
    }
    vec
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Move")]
pub enum MoveDef {
    Normal {
        #[serde(with = "RoleDef")]
        role: Role,
        #[serde(with = "SquareDef")]
        from: Square,
        #[serde(with = "opt_external_struct")]
        capture: Option<Role>,
        #[serde(with = "SquareDef")]
        to: Square,
        #[serde(with = "opt_external_struct")]
        promotion: Option<Role>,
    },
    EnPassant {
        #[serde(with = "SquareDef")]
        from: Square,
        #[serde(with = "SquareDef")]
        to: Square,
    },
    Castle {
        #[serde(with = "SquareDef")]
        king: Square,
        #[serde(with = "SquareDef")]
        rook: Square,
    },
    Put {
        #[serde(with = "RoleDef")]
        role: Role,
        #[serde(with = "SquareDef")]
        to: Square,
    },
}

#[derive(Serialize)]
pub struct MoveWrapper<'a>(#[serde(with = "MoveDef")] &'a Move);

impl<'a> MoveWrapper<'a> {
    pub fn new(chess_move: &'a Move) -> Self {
        MoveWrapper(chess_move)
    }
}

impl ChessGame {
    // Creates a new chess game with the default position
    pub fn new() -> Self {
        ChessGame {
            game: Chess::default(),
        }
    }

    // Makes a move, if it is legal
    pub fn make_move(&mut self, m: &Move) {
        self.game.play_unchecked(m);
    }

    // Returns a list of legal moves
    pub fn get_legal_moves(&self) -> MoveList {
        self.game.legal_moves()
    }

    // Returns the FEN representation of the current position
    pub fn fen(&self) -> String {
        self.game.board().to_string()
    }

    // Returns the UCI representation of the legal moves
    pub fn uci(&self) -> Vec<String> {
        self.get_legal_moves()
            .iter()
            .map(|m| m.to_string())
            .collect()
    }
}
