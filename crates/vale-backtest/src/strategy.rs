use crate::context::Context;
use vale_core::types::Bar;

pub trait Strategy: Send {
    fn name(&self) -> &str;
    fn on_start(&mut self, _ctx: &mut Context) {}
    fn on_bar(&mut self, ctx: &mut Context, bar: &Bar);
    fn on_end(&mut self, _ctx: &mut Context) {}
}
