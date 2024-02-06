use crate::game::*;

#[derive(Debug, Clone)]
pub struct PileMappings {
    card_id_to_index: Vec<usize>,
    index_to_card_id: Vec<CardId>,
}

impl PileMappings {
    pub fn new(pile: &Pile) -> Self {
        assert!(pile.len() > 0);
        assert!(pile.len() <= 9);

        let max_num = pile.iter().map(|c| c.get_card_id()).max().unwrap();

        let mut card_id_to_index = vec![0; (max_num + 1) as usize];
        let mut index_to_card_id = vec![0; pile.len()];

        for (i, c) in pile.iter().enumerate() {
            card_id_to_index[c.get_card_id() as usize] = i;
            index_to_card_id[i] = c.get_card_id();
        }

        Self {
            card_id_to_index,
            index_to_card_id,
        }
    }

    pub fn index(&self, card_id: CardId) -> usize {
        self.card_id_to_index[card_id as usize]
    }
    pub fn card_id(&self, index: usize) -> CardId {
        self.index_to_card_id[index]
    }
}
