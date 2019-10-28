use actix::prelude::*;
use actix_web_actors::ws;

pub struct WebSocketClient;

impl Actor for WebSocketClient {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.add_self(ctx);
    }
    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.remove_self(ctx);
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WebSocketClient {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Close(_) => {
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl WebSocketClient {
    pub fn new() -> Self {
        Self
    }

    fn add_self(&self, ctx: &mut <Self as Actor>::Context) {
        let _own_address = ctx.address().clone();
        //Todo Send own address to tracking service
    }

    fn remove_self(&self, ctx: &mut <Self as Actor>::Context) {
        let _own_address = ctx.address().clone();
        //Todo Remove own address from tracking service
    }
}
