//! TODO!

mod board;
mod sim;
mod sim_rpc;
mod websocket;

pub use board::*;
pub use sim::*;
pub use sim_rpc::*;
pub use websocket::*;

#[derive(Debug)]
pub enum BlackBox<O = ()> {
    Board(BoardStorage),
    Sim(SimStorage),
    SimWithRpc(SimWithRpcStorage),
    WebSocket(WebSocketStorage),
    Other(O),
    Empty,
}

impl<O> BlackBox<O> {
    pub fn new() -> Self {
        BlackBox::Empty
    }
}
