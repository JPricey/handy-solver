// TOOD: this
// #[derive(Copy, Clone, PartialEq, Eq, Hash)]
// pub struct CardPtrAsRef {
//     pub card_def: &'static CardDef,
//     pub key: FaceKey,
// }
// 
// impl CardPtrAsRef {
//     pub fn new_from_id(id: usize, key: FaceKey) -> CardPtrAsRef {
//         CardPtrAsRef {
//             card_def: CARDS.get_card(id),
//             key,
//         }
//     }
// 
//     pub fn new_from_id_u8(id: u8, key: FaceKey) -> CardPtrAsRef {
//         CardPtrAsRef {
//             card_def: CARDS.get_card(id as usize),
//             key,
//         }
//     }
// 
//     pub fn get_card_id(&self) -> CardId {
//         self.card_def.id
//     }
// 
//     pub fn get_card_face(&self) -> FaceKey {
//         self.key
//     }
// 
//     pub fn get_card_def(&self) -> &CardDef {
//         &self.card_def
//     }
// 
//     pub fn get_active_face(&self) -> &FaceDef {
//         &self.card_def.faces[self.key]
//     }
// }
// 
// impl fmt::Debug for CardPtrAsRef {
//     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
//         let string = format!("{}{}", self.get_card_id(), self.key);
//         let colored = match self.get_active_face().allegiance {
//             Allegiance::Hero => string.blue(),
//             Allegiance::Monster => string.red(),
//             Allegiance::Werewolf => string.yellow(),
//         };
//         write!(fmt, "{}", colored)
//     }
// }
// 
