use actix::prelude::*;
use actix_web_actors::ws;

use crate::http_server::SharedState;

pub struct MyWebSocket {
    pub connections: SharedState,
}

impl MyWebSocket {
    pub fn new(connections: SharedState) -> Self {
        Self { connections }
    }

    fn send_message(&self, ctx: &mut <Self as Actor>::Context, message: &str) {
        ctx.text(message);
    }
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.connections.write().unwrap().push(addr);
    }
}

// How we are supposed to respond to incoming WebSocket Messages, for now
// just ignore
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        log::debug!("WS: {msg:?}");
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

// Define messages for inter-thread communication
#[derive(Clone)]
pub struct Notification(pub String);

impl Message for Notification {
    type Result = ();
}

impl Handler<Notification> for MyWebSocket {
    type Result = ();

    fn handle(&mut self, msg: Notification, ctx: &mut Self::Context) {
        self.send_message(ctx, &msg.0);
    }
}
