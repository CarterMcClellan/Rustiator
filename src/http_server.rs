use actix::Addr;
use actix_web_actors::ws;
use std::sync::mpsc;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};
use tokio::task::JoinSet;

use actix_cors::Cors;
use actix_files as fs;
use actix_web::{
    get, http, middleware, post, web, web::Json, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};

use log::{error, info};

use dashmap::DashMap;
use handlebars::Handlebars;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use serde_json::json;
use uuid::Uuid;

use shakmaty::uci::Uci;

use crate::chess_engine::{self, RandomEngine, ToChooseMove};
use crate::player_vs_bot::PlayerGame;
use crate::websocket::MyWebSocket;
use crate::{chess_engine::engine_vs_engine, chess_game::ChessGame, lua::StatelessLuaBot};

pub type GameMap = DashMap<Uuid, Arc<RwLock<ChessGame>>>;
pub type Connection = Addr<MyWebSocket>;
pub type SharedState = Arc<RwLock<Vec<Connection>>>;
// TODO: this is actually incomplete. We will need to store saved bots as more like
// instructions to create a new bot. We need to distinguish between a serialized bot and
// and a deserialized bot. In case the bots are stateful
pub type SavedBots = DashMap<String, Box<dyn ToChooseMove + Send + Sync>>;
pub type SavedLuaBots = DashMap<String, StatelessLuaBot>;

#[derive(Deserialize, Debug)]
struct NewGameArgs {
    mode: String,
    #[serde(rename = "botName")]
    bot_name: String,
}

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().body("pong".to_string())
}

#[post("/new_game")]
async fn new_game(
    app_data: web::Data<GlobalAppData>,
    req_body: Json<NewGameArgs>,
) -> impl Responder {
    info!("recieved request!");
    // Generate a new UUID
    let new_game_id = Uuid::new_v4();

    match req_body.mode.as_str() {
        "playerVsBot" => {
            // TODO: allow bot id in request body to select bot to play here
            let Some(saved_bot) = app_data.saved_bots.get(&req_body.bot_name) else {
                return HttpResponse::BadRequest()
                    .body(format!("No bot named {} found", &req_body.bot_name));
            };
            let game = PlayerGame::new(saved_bot.to_choose_move(), &req_body.bot_name);
            info!("Starting Player vs Bot Game: {new_game_id}");
            app_data.active_player_games.insert(new_game_id, game);
        }
        "botVsBot" => {
            let game = Arc::new(RwLock::new(ChessGame::new()));
            // TODO: add option for second bot so player can select which 2 bots should play each other
            let engine1 = Arc::new(RandomEngine::new());
            let engine2 = Arc::new(RandomEngine::new());

            let game_clone = game.clone();
            let engine1_clone = engine1.clone();
            let engine2_clone = engine2.clone();

            let (tx, rx) = mpsc::channel();

            let mut game_join_set = JoinSet::new();

            game_join_set.spawn_blocking(move || {
                engine_vs_engine(game_clone, engine1_clone, engine2_clone, tx);
            });

            let new_game_connections: SharedState = Arc::new(RwLock::new(Vec::new()));
            app_data
                .connections
                .insert(new_game_id, new_game_connections.clone());
            game_join_set.spawn_blocking(move || {
                loop {
                    let result = match rx.recv() {
                        Ok(message) => message,
                        Err(e) => {
                            error!("Game over, terminating the thread: {:?}", e);
                            break; // Exit the loop and end the thread
                        }
                    };

                    let connections_clone = new_game_connections.read().unwrap();
                    for conn in connections_clone.iter() {
                        conn.do_send(result.clone());
                    }
                }
            });

            // not doing anything with set right now, but save in case in the future
            // we want to do some graceful shutdown logic
            let mut active_tasks = app_data.active_processes.lock().unwrap();
            active_tasks.insert(new_game_id, game_join_set);

            info!("inserted game {} into active tasks", new_game_id);
            app_data.active_bot_bot_games.insert(new_game_id, game);
        }
        _ => {
            return HttpResponse::BadRequest().body("Invalid game mode");
        }
    }

    // Return the new game ID to the client
    info!("reached return ");
    HttpResponse::Ok().json(serde_json::json!({ "game_id": new_game_id.to_string() }))
}

#[get("/spectate/{uuid}")]
async fn spectate_game(
    app_data: web::Data<GlobalAppData>,
    hb: web::Data<Handlebars<'_>>,
    info: web::Path<Uuid>,
) -> impl Responder {
    let game_uuid = info.into_inner();

    // Fetch the game data
    let game_data = match app_data.active_bot_bot_games.get(&game_uuid) {
        Some(game) => game,
        None => return HttpResponse::NotFound().body("Game not found"),
    };

    let gd_lock = match game_data.read() {
        Ok(gd) => gd,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to lock game data"),
    };

    let position = gd_lock.fen();
    let css_content = std::fs::read_to_string("./client/css/chessboard-1.0.0.min.css").unwrap();
    let js_content = std::fs::read_to_string("./client/js/chessboard-1.0.0.js").unwrap();

    // Create data to fill the template
    let data = json!({
        "game_id": game_uuid.to_string(),
        "position": position,
        "style": css_content,
        "board_js":js_content
    });

    // Render the template with the data
    let body = hb.render("spectate_template", &data).unwrap_or_else(|err| {
        error!("Template rendering error: {}", err);
        "Template rendering error".to_string()
    });

    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    uuid: web::Path<Uuid>, // Extract UUID from the path
    app_data: web::Data<GlobalAppData>,
) -> Result<HttpResponse, Error> {
    info!("New Connection to Game: {}", &uuid);
    match app_data.connections.get(&uuid) {
        Some(game_conns) => {
            let game_conns: SharedState = game_conns.clone();
            let ws = MyWebSocket::new(game_conns);
            ws::start(ws, &req, stream)
        }
        None => {
            let err_msg = format!("Room {} not found", &uuid);
            let err = std::io::Error::new(std::io::ErrorKind::NotFound, err_msg);
            return Err(err.into());
        }
    }
}

#[derive(Deserialize, Debug)]
struct PlayGameArgs {
    #[serde(rename = "move", deserialize_with = "parse_uci")]
    /// Move in UCI notation
    player_move: Uci,
}

fn parse_uci<'de, D>(deserializer: D) -> Result<Uci, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: &[u8] = Deserialize::deserialize(deserializer)?;
    Uci::from_ascii(bytes).map_err(D::Error::custom)
}

#[derive(Serialize, Debug)]
struct PlayGameResponse {
    /// State in fen representation
    board_state: String,
}

#[post("/play/{uuid}")]
/// Play a given move against a bot
pub async fn player_vs_bot(
    app_data: web::Data<GlobalAppData>,
    req_body: Json<PlayGameArgs>,
    uuid: web::Path<Uuid>,
) -> actix_web::Result<Json<PlayGameResponse>> {
    let Some(mut game) = app_data.active_player_games.get_mut(&uuid) else {
        return Err(actix_web::error::ErrorBadRequest(format!(
            "No active game for {uuid}"
        )));
    };

    log::debug!("Player Trying to play: {}", req_body.player_move);

    let player_move = req_body
        .player_move
        .to_move(&game.game.game /*lmao*/)
        .map_err(|e| {
            actix_web::error::ErrorBadRequest(format!(
                "Error Playing Move {}: {e}",
                req_body.player_move
            ))
        })?;

    match game.play_move(player_move) {
        Ok(_) => {}
        Err(e) => {
            error!("Error playing move: {e:?}");
            return Err(actix_web::error::ErrorInternalServerError(format!(
                "Error Playing Move {}: {e:?}",
                req_body.player_move
            )));
        }
    }

    Ok(Json(PlayGameResponse {
        board_state: game.fen(),
    }))
}

#[get("/game/{uuid}")]
async fn play_game_entry(
    app_data: web::Data<GlobalAppData>,
    hb: web::Data<Handlebars<'_>>,
    uuid: web::Path<Uuid>,
) -> impl Responder {
    let Some(game) = app_data.active_player_games.get(&uuid) else {
        return Err(actix_web::error::ErrorBadRequest(format!(
            "No active game for {uuid}"
        )));
    };

    let css_content = std::fs::read_to_string("./client/css/chessboard-1.0.0.min.css").unwrap();
    let js_content = std::fs::read_to_string("./client/js/chessboard-1.0.0.js").unwrap();

    // Create data to fill the template
    let data = json!({
        "game_id": uuid.to_string(),
        "position": game.fen(),
        "style": css_content,
        "board_js":js_content,
        "bot_name": game.bot_name,
    });

    // Render the template with the data
    let body = hb.render("game_template", &data).unwrap_or_else(|err| {
        error!("Template rendering error: {}", err);
        "Template rendering error".to_string()
    });

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[derive(Deserialize)]
struct NewBotRequest {
    script: String,
    #[serde(rename = "botName")]
    bot_name: String,
}

#[derive(Serialize)]
struct NewBotResponse {}

#[post("/newBot")]
async fn new_bot(
    request: Json<NewBotRequest>,
    app_data: web::Data<GlobalAppData>,
) -> actix_web::Result<Json<NewBotResponse>> {
    let request = request.0;
    let bot = StatelessLuaBot::try_new(request.script)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Failed to initialize Bot: {e}")))?;

    // TODO: for now this will overwrite other saved bots of the same name.
    // I think this is what we want for now. But later we will need to block this and
    // then add an ability to edit already saved bots
    app_data
        .saved_bots
        .insert(request.bot_name.clone(), Box::new(bot.clone()));
    app_data.saved_lua_bots.insert(request.bot_name, bot);

    Ok(Json(NewBotResponse {}))
}

// #[get("/editBot/{bot_name}")]
async fn edit_bot(
    request: actix_web::HttpRequest,
    hb: web::Data<Handlebars<'_>>,
) -> impl Responder {
    // This will be "" in the case where they are making a new bot
    let bot_name = request.match_info().query("bot_name");
    // Create data to fill the template
    let data = json!({
        "bot_name": bot_name,
    });

    // Render the template with the data
    let body = hb.render("editor_template", &data).unwrap_or_else(|err| {
        error!("Template rendering error: {}", err);
        "Template rendering error".to_string()
    });

    HttpResponse::Ok().content_type("text/html").body(body)
}

#[get("script/{bot_name}")]
async fn get_script(
    bot_name: web::Path<String>,
    app_data: web::Data<GlobalAppData>,
) -> impl Responder {
    let script = app_data
        .saved_lua_bots
        .get(&bot_name.to_string())
        .map(|bot| bot.script.clone())
        .unwrap_or_else(|| {
            log::warn!(
                r#"Tried to get script for nonexistant bot "{bot_name}" defaulting to template"#
            );
            include_str!("../client/template.lua").to_string()
        });
    script
}

pub struct GlobalAppData {
    active_processes: Arc<Mutex<HashMap<Uuid, JoinSet<()>>>>,
    active_player_games: DashMap<Uuid, PlayerGame>,
    connections: DashMap<Uuid, SharedState>,
    active_bot_bot_games: GameMap,
    saved_bots: SavedBots,
    saved_lua_bots: SavedLuaBots,
}

pub async fn start_server(hostname: String, port: u16) -> std::io::Result<()> {
    // Init an empty hashmap to store all the ongoing processes
    let active_processes = Arc::new(Mutex::new(HashMap::<Uuid, JoinSet<()>>::new()));

    let active_player_games = DashMap::<Uuid, PlayerGame>::new();

    // Initialize an empty hashmap which maps UUID to ChessGame
    let active_bot_bot_games: GameMap = DashMap::new();

    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file("spectate_template", "./client/spectate.html")
        .unwrap(); // lol fix
    handlebars
        .register_template_file("game_template", "./client/game.html")
        .unwrap(); // lmao fix
    handlebars
        .register_template_file("editor_template", "./client/text_editor.html")
        .unwrap(); // lol don't fix?
    let handlebars_ref = web::Data::new(handlebars);

    // Active Spectator connections
    let connections: DashMap<Uuid, SharedState> = DashMap::new();

    let saved_bots: SavedBots = DashMap::new();
    saved_bots.insert(String::from("RustRandomBot"), Box::new(RandomEngine::new()));

    let saved_lua_bots = DashMap::new();

    let app_data = web::Data::new(GlobalAppData {
        connections,
        active_processes,
        active_bot_bot_games,
        active_player_games,
        saved_bots,
        saved_lua_bots,
    });

    info!("Starting server on {}:{}", hostname, port);
    let allowed_origin = format!("http://{}:{}", &hostname, &port);
    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allowed_origin(&allowed_origin)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        if port == 80 {
            cors = cors.allowed_origin(&format!("http://{hostname}"))
        }

        // because we are doing a wildcard match with render game, the order of the
        // routes actually does matter here
        App::new()
            .wrap(cors)
            .app_data(app_data.clone()) // Add the shared state to the app
            .app_data(handlebars_ref.clone())
            .route("/ws/{uuid}", web::get().to(ws_index))
            .service(spectate_game)
            .service(new_game)
            .service(player_vs_bot)
            .service(play_game_entry)
            .service(new_bot)
            .service(get_script)
            .service(web::resource(["/editBot", "/editBot/{bot_name}"]).to(edit_bot))
            .service(fs::Files::new("/", "./client/").index_file("index.html"))
            // .service(fs::Files::new("/img", "./client/img"))
            .service(
                web::scope("/img")
                    .wrap(
                        middleware::DefaultHeaders::new()
                            .header("Cache-Control", "public, max-age=86400"),
                    )
                    .service(fs::Files::new("", "./client/img").use_last_modified(true)),
            )
    })
    .workers(4) // Set the number of worker threads
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
