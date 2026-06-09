use super::MessageFilter;
use rand::Rng;
use serenity::all::{Context, Message};
use std::sync::Arc;

/// Passes with the given probability in `[0.0, 1.0]`.
/// A probability of 1.0 always passes; 0.0 never passes.
pub fn chance(probability: f64) -> Arc<dyn MessageFilter> {
    Arc::new(ChanceFilter(probability))
}

struct ChanceFilter(f64);

impl MessageFilter for ChanceFilter {
    fn check(&self, _ctx: &Context, _msg: &Message) -> bool {
        rand::thread_rng().gen::<f64>() < self.0
    }
}
