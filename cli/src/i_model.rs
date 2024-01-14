use crate::game::Pile

trait ModelT {
     fn score_pile(&self, pile: &Pile) -> f32;
}
