use crate::game::*;
use crate::solver::pile_mapping::*;
use std::cmp::Ord;
use std::fmt::Debug;

pub trait StorablePileT: Ord + PartialEq + Eq + Clone {}
impl<T> StorablePileT for T where T: Ord + PartialEq + Eq + Clone {}

pub trait PileStorageConverter<T: StorablePileT> {
    fn new_from_pile(pile: &Pile) -> Self;
    fn pile_to_tiny_pile(&self, pile: &Pile) -> T;
    fn tiny_pile_to_pile(&self, tiny_pile: &T) -> Pile;
}

pub struct NoopPileStorageConverter {}
impl PileStorageConverter<Pile> for NoopPileStorageConverter {
    fn new_from_pile(_pile: &Pile) -> Self {
        NoopPileStorageConverter {}
    }
    fn pile_to_tiny_pile(&self, pile: &Pile) -> Pile {
        pile.clone()
    }
    fn tiny_pile_to_pile(&self, pile: &Pile) -> Pile {
        pile.clone()
    }
}

type PackedFaces = [u8; 3];

type OrderingType = [usize; 9];
type TinyPileOrderingType = u32;

pub struct TinyPileConverter {
    pile_mappings: PileMappings,
}

// http://antoinecomeau.blogspot.com/2014/07/mapping-between-permutations-and.html
fn number_to_ordering(number: TinyPileOrderingType) -> OrderingType {
    let n = 9;
    let mut result = OrderingType::default();
    let mut elems: OrderingType = [0, 1, 2, 3, 4, 5, 6, 7, 8];

    let mut ind;
    let mut m = number as usize;
    for i in 0..n {
        ind = m % (n - i);
        m = m / (n - i);
        result[i] = elems[ind];
        elems[ind] = elems[n - i - 1];
    }
    result
}

fn order_to_number(perm: &OrderingType) -> TinyPileOrderingType {
    let n = 9;
    let mut k = 0;
    let mut m = 1;
    let mut pos = [0, 1, 2, 3, 4, 5, 6, 7, 8];
    let mut elems = [0, 1, 2, 3, 4, 5, 6, 7, 8];

    for i in 0..8 {
        k += m * pos[perm[i]];
        m = m * (n - i);
        pos[elems[n - i - 1]] = pos[perm[i]];
        elems[pos[perm[i]]] = elems[n - i - 1];
    }

    k as TinyPileOrderingType
}

impl TinyPileConverter {
    fn pile_to_order(&self, pile: &Pile) -> OrderingType {
        [
            self.pile_mappings.index(pile[0].get_card_id()),
            self.pile_mappings.index(pile[1].get_card_id()),
            self.pile_mappings.index(pile[2].get_card_id()),
            self.pile_mappings.index(pile[3].get_card_id()),
            self.pile_mappings.index(pile[4].get_card_id()),
            self.pile_mappings.index(pile[5].get_card_id()),
            self.pile_mappings.index(pile[6].get_card_id()),
            self.pile_mappings.index(pile[7].get_card_id()),
            self.pile_mappings.index(pile[8].get_card_id()),
        ]
    }

    fn index_to_card_id(&self, index: usize) -> CardId {
        self.pile_mappings.card_id(index)
    }
}

impl PileStorageConverter<TinyPile> for TinyPileConverter {
    fn new_from_pile(pile: &Pile) -> Self {
        Self {
            pile_mappings: PileMappings::new(pile),
        }
    }

    fn pile_to_tiny_pile(&self, pile: &Pile) -> TinyPile {
        TinyPile {
            pile_key: order_to_number(&self.pile_to_order(&pile)),
            faces: pack_faces_from_pile(pile),
        }
    }

    fn tiny_pile_to_pile(&self, tiny_pile: &TinyPile) -> Pile {
        let ordering = number_to_ordering(tiny_pile.pile_key);
        let mut result = Pile::default();
        result.push(CardPtr::new_from_id(
            self.index_to_card_id(ordering[0]),
            face_key_from_byte(tiny_pile.faces[0], 0),
        ));
        result.push(CardPtr::new_from_id(
            self.index_to_card_id(ordering[1]),
            face_key_from_byte(tiny_pile.faces[0], 1),
        ));
        result.push(CardPtr::new_from_id(
            self.index_to_card_id(ordering[2]),
            face_key_from_byte(tiny_pile.faces[0], 2),
        ));
        result.push(CardPtr::new_from_id(
            self.index_to_card_id(ordering[3]),
            face_key_from_byte(tiny_pile.faces[0], 3),
        ));
        result.push(CardPtr::new_from_id(
            self.index_to_card_id(ordering[4]),
            face_key_from_byte(tiny_pile.faces[1], 0),
        ));
        result.push(CardPtr::new_from_id(
            self.index_to_card_id(ordering[5]),
            face_key_from_byte(tiny_pile.faces[1], 1),
        ));
        result.push(CardPtr::new_from_id(
            self.index_to_card_id(ordering[6]),
            face_key_from_byte(tiny_pile.faces[1], 2),
        ));
        result.push(CardPtr::new_from_id(
            self.index_to_card_id(ordering[7]),
            face_key_from_byte(tiny_pile.faces[1], 3),
        ));
        result.push(CardPtr::new_from_id(
            self.index_to_card_id(ordering[8]),
            face_key_from_byte(tiny_pile.faces[2], 3),
        ));

        result
    }
}

fn face_key_from_byte(byte: u8, index: usize) -> FaceKey {
    match (byte >> (3 - index) * 2) as u8 & 0b00000011 {
        0 => FaceKey::A,
        1 => FaceKey::B,
        2 => FaceKey::C,
        3 => FaceKey::D,
        _ => panic!(),
    }
}

fn face_to_u8(face: FaceKey) -> u8 {
    match face {
        FaceKey::A => 0,
        FaceKey::B => 1,
        FaceKey::C => 2,
        FaceKey::D => 3,
    }
}

fn pack_faces_from_pile(pile: &Pile) -> PackedFaces {
    let first: u8 = (face_to_u8(pile[0].get_card_face()) << 6)
        | (face_to_u8(pile[1].get_card_face()) << 4)
        | (face_to_u8(pile[2].get_card_face()) << 2)
        | (face_to_u8(pile[3].get_card_face()) << 0);
    let second: u8 = (face_to_u8(pile[4].get_card_face()) << 6)
        | (face_to_u8(pile[5].get_card_face()) << 4)
        | (face_to_u8(pile[6].get_card_face()) << 2)
        | (face_to_u8(pile[7].get_card_face()) << 0);
    let third: u8 = face_to_u8(pile[8].get_card_face());

    [first, second, third]
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct TinyPile {
    pile_key: u32,
    faces: [u8; 3],
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::string_to_pile;

    #[test]
    fn test_face_pack() {
        assert_eq!(
            pack_faces_from_pile(&string_to_pile("1A 1A 1A 1A 1A 1A 1A 1A 1A")),
            [0, 0, 0] as PackedFaces
        );

        let ex2 = pack_faces_from_pile(&string_to_pile("1A 1B 1C 1D 1D 1C 1B 1A 1D"));
        assert_eq!(ex2, [0b00011011, 0b11100100, 0b00000011] as PackedFaces);

        assert_eq!(face_key_from_byte(ex2[0], 0), FaceKey::A);
        assert_eq!(face_key_from_byte(ex2[0], 1), FaceKey::B);
        assert_eq!(face_key_from_byte(ex2[0], 2), FaceKey::C);
        assert_eq!(face_key_from_byte(ex2[0], 3), FaceKey::D);

        assert_eq!(face_key_from_byte(ex2[1], 0), FaceKey::D);
        assert_eq!(face_key_from_byte(ex2[1], 1), FaceKey::C);
        assert_eq!(face_key_from_byte(ex2[1], 2), FaceKey::B);
        assert_eq!(face_key_from_byte(ex2[1], 3), FaceKey::A);

        assert_eq!(face_key_from_byte(ex2[2], 3), FaceKey::D);
    }

    #[test]
    fn test_converter() {
        let base_pile = string_to_pile("1A 2A 3A 4A 5A 6A 7A 8A 9A");
        let converter = TinyPileConverter::new_from_pile(&base_pile);

        {
            let pile = base_pile.clone();
            let tiny_pile = converter.pile_to_tiny_pile(&pile);
            let reformed_pile = converter.tiny_pile_to_pile(&tiny_pile);
            assert_eq!(pile, reformed_pile);
        }

        {
            let pile = string_to_pile("9D 8B 5A 4B 1A 6A 2B 7C 3D");
            let tiny_pile = converter.pile_to_tiny_pile(&pile);
            let reformed_pile = converter.tiny_pile_to_pile(&tiny_pile);
            assert_eq!(pile, reformed_pile);
        }
    }

    #[test]
    fn test_converter_all_numbers() {
        let base_pile = string_to_pile("1A 2A 3A 4A 5A 6A 7A 8A 9A");
        let converter = TinyPileConverter::new_from_pile(&base_pile);

        for i in 0..362880 {
            let tiny_pile = TinyPile {
                pile_key: i,
                faces: [0; 3],
            };

            let pile = converter.tiny_pile_to_pile(&tiny_pile);
            let result = converter.pile_to_tiny_pile(&pile);

            assert_eq!(tiny_pile, result);
        }
    }
}
