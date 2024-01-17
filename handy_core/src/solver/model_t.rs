use crate::game::Pile;

pub trait ModelT {
    fn score_pile(&self, pile: &Pile) -> f32;
}
