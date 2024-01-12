use mlua::{Lua, StdLib};

use anyhow::Result as AnyResult;

use shakmaty::MoveList;

use tokio::sync::mpsc;


pub struct LuaBot<'lua> {
    vm: Lua,
    bot: mlua::Table<'lua>,
}

/*
 * Ideally a LuaBot would look like this
 * ```rust
 * pub struct LuaBot<'lua> {
 *     vm: Lua,
 *     bot: mlua::Table<'lua>,
 * }
 * ```
 * 
 * Where we hold the initialized bot and the lua vm that it is running in.
 * However this is a self referential struct which is impossible in safe rust.
 * 
 * 
 * Instead, going to try a hack where we spawn the vm in a tokio task and then communicate with it via a channel
 */



// impl<'lua> LuaBot<'lua> {
//     pub fn try_new(script: &str) -> AnyResult<Self>{
//         let vm = init_vm();
//         let chunk = vm.load(script);
//         let bot = chunk.eval()?;

//         Ok(Self {
//             vm, bot
//         })
//     }
// }

fn spawn_vm(script: String, rcv: mpsc::Receiver<()>, sender: mpsc::Sender<AnyResult<String>>) -> AnyResult<()> {
    
    tokio::spawn(async move {
        let vm = init_vm();
        let chunk = vm.load(script);
        let bot: mlua::Table = chunk.eval().unwrap();
    
        //TODO: validate bot.
        // 1. make sure it has chooseMove function and passing in a simple example works.
        let vm = vm;

        while let Some(args) = rcv.recv().await {
            let chosen_move = choose_move(bot.clone(), "TODO:", &[String::new()]);

            sender.send(chosen_move).await;
        }
    });

    Ok(())
}

fn choose_move<'lua>(bot: mlua::Table<'lua>, chess_game: &str, legal_moves: &[String]) -> AnyResult<String> {
    let choose_move: mlua::Function = bot.get("chooseMove")?;
    let chosen_move = choose_move.call((bot, chess_game, legal_moves))?;
    Ok(chosen_move)
}

fn init_vm() -> Lua {
    // the Anti-Eamonn policy
    let whitelisted_libs = StdLib::MATH & StdLib::TABLE & StdLib::STRING;
    // SAFETY: This function only errors if we use StdLib::DEBUG or StdLib::FFI
    let lua = Lua::new_with(whitelisted_libs, mlua::LuaOptions::default()).unwrap();
    lua
}