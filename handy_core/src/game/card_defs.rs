use crate::game::primitives::*;
use enum_map::EnumMap;
use std::cmp;

use enum_map::enum_map;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CARDS: CardDefs = CardDefs::new();
    pub static ref ROW_PUSH_ALLY_INF_PULL_ALLY_INF: Row = row().push_ally_inf().pull_ally_inf();
    pub static ref ROW_REVIVE_ALLY: Row = row().revive_ally();
    pub static ref ROW_CLAW_ODD_ENEMIES: Row = row().claw_odd_enemies();
    pub static ref ROW_CLAW_EVEN_ENEMIES: Row = row().claw_even_enemies();
    pub static ref ROW_PUSH_ENEMY_INF_PUSH_ENEMY_INF: Row = row().push_enemy_inf().push_enemy_inf();
    pub static ref ROW_CLAW_ENEMY_4: Row = row().claw_enemy(4);
    pub static ref ROW_HEAL_ALLY: Row = row().heal_ally();
    pub static ref ROW_HEAL_ALLY_CLAW_ENEMY_3: Row = row().heal_ally().claw_enemy(3);
}

struct CharBuilder {
    pub class: Class,
    pub allegiance: Allegiance,
}

impl CharBuilder {
    fn new(class: Class, allegiance: Allegiance) -> CharBuilder {
        CharBuilder { class, allegiance }
    }

    fn card(&self, id: CardId, mut faces: EnumMap<FaceKey, FaceDef>) -> CardDef {
        for face in faces.values_mut() {
            match face.allegiance {
                Allegiance::Rat | Allegiance::Werewolf => {
                    // Allegiance is correct already
                }
                Allegiance::Hero | Allegiance::Baddie => {
                    // Overwrite
                    face.allegiance = self.allegiance
                }
            }
        }
        CardDef {
            id,
            faces,
            class: self.class,
            is_back_start: false,
        }
    }

    fn back_card(&self, id: CardId, mut faces: EnumMap<FaceKey, FaceDef>) -> CardDef {
        for face in faces.values_mut() {
            if face.allegiance != Allegiance::Werewolf {
                face.allegiance = self.allegiance
            }
        }
        CardDef {
            id,
            faces,
            class: self.class,
            is_back_start: true,
        }
    }
}

fn row() -> Row {
    Row {
        is_mandatory: false,
        condition: None,
        mandatory: None,
        actions: vec![],
    }
}

impl Row {
    fn add_condition(mut self, condition: Condition) -> Self {
        self.condition = Some(condition);
        self
    }

    fn rage_condition(mut self, rage: ConditionCountType) -> Self {
        self.condition = Some(Condition::Rage(rage));
        self
    }

    fn energy_cost(mut self, cost: ConditionCountType) -> Self {
        self.condition = Some(Condition::Cost(ConditionCostType::Energy, cost));
        self
    }

    fn dodge_cost(mut self, cost: ConditionCountType) -> Self {
        self.condition = Some(Condition::Cost(ConditionCostType::Dodge, cost));
        self
    }

    fn add_mandatory(mut self, mandatory: SelfAction) -> Self {
        self.mandatory = Some(mandatory);
        self
    }

    fn rotate(self) -> Self {
        self.add_mandatory(SelfAction::Rotate)
    }

    fn flip(self) -> Self {
        self.add_mandatory(SelfAction::Flip)
    }

    fn manouver(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Manouver,
            target: Target::Ally,
        });
        self
    }

    fn arrow_any(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Arrow,
            target: Target::Any,
        });
        self
    }

    fn double_arrow_any(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::DoubleArrow,
            target: Target::Any,
        });
        self
    }

    fn delay_ally(mut self, amount: usize) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Delay(amount),
            target: Target::Ally,
        });
        self
    }

    fn delay_any(mut self, range: usize) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Delay(range),
            target: Target::Any,
        });
        self
    }

    fn delay_enemy(mut self, range: usize) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Delay(range),
            target: Target::Enemy,
        });
        self
    }

    fn quicken_any(mut self, range: usize) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Quicken(range),
            target: Target::Any,
        });
        self
    }

    fn quicken_enemy(mut self, range: usize) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Quicken(range),
            target: Target::Enemy,
        });
        self
    }

    fn quicken_ally(mut self, range: usize) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Quicken(range),
            target: Target::Ally,
        });
        self
    }

    fn hit_any(mut self, range: usize) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Hit(Range::Int(range)),
            target: Target::Any,
        });
        self
    }

    fn hit_any_inf(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Hit(Range::Inf),
            target: Target::Any,
        });
        self
    }

    fn hit_ally(mut self, range: usize) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Hit(Range::Int(range)),
            target: Target::Ally,
        });
        self
    }

    fn hit_ally_inf(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Hit(Range::Inf),
            target: Target::Ally,
        });
        self
    }

    fn heal_any(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Heal,
            target: Target::Any,
        });
        self
    }

    fn heal_ally(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Heal,
            target: Target::Ally,
        });
        self
    }

    fn heal_enemy(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Heal,
            target: Target::Enemy,
        });
        self
    }

    fn revive_ally(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Revive,
            target: Target::Ally,
        });
        self
    }

    fn revive_any(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Revive,
            target: Target::Any,
        });
        self
    }

    fn hit_enemy(mut self, range: usize) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Hit(Range::Int(range)),
            target: Target::Enemy,
        });
        self
    }

    fn hit_enemy_inf(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Hit(Range::Inf),
            target: Target::Enemy,
        });
        self
    }

    fn pull_enemy(mut self, range: usize) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Pull(Range::Int(range)),
            target: Target::Enemy,
        });
        self
    }

    fn pull_enemy_inf(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Pull(Range::Inf),
            target: Target::Enemy,
        });
        self
    }

    fn pull_ally(mut self, range: usize) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Pull(Range::Int(range)),
            target: Target::Ally,
        });
        self
    }

    fn pull_ally_inf(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Pull(Range::Inf),
            target: Target::Ally,
        });
        self
    }

    fn push_enemy(mut self, range: usize) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Push(Range::Int(range)),
            target: Target::Enemy,
        });
        self
    }

    fn push_enemy_inf(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Push(Range::Inf),
            target: Target::Enemy,
        });
        self
    }

    fn push_ally_inf(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Push(Range::Inf),
            target: Target::Ally,
        });
        self
    }

    fn inspire_ally(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Inspire,
            target: Target::Ally,
        });
        self
    }

    fn inspire_enemy(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Inspire,
            target: Target::Enemy,
        });
        self
    }

    fn void(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Void,
            target: Target::Enemy,
        });
        self
    }

    fn death(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Death,
            target: Target::Enemy,
        });
        self
    }

    fn claw_enemy(mut self, range: usize) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Claws(Range::Int(range)),
            target: Target::Enemy,
        });
        self
    }

    fn claw_enemy_inf(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Claws(Range::Inf),
            target: Target::Enemy,
        });
        self
    }

    fn claw_odd_enemies(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::SpacedClaws(ClawSpaceType::Odd),
            target: Target::Enemy,
        });
        self
    }

    fn claw_even_enemies(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::SpacedClaws(ClawSpaceType::Even),
            target: Target::Enemy,
        });
        self
    }

    fn fireball(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Fireball,
            target: Target::Any,
        });
        self
    }

    fn ablaze(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Ablaze,
            target: Target::Any,
        });
        self
    }

    fn teleport_ally(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Teleport,
            target: Target::Ally,
        });
        self
    }

    fn teleport_enemy(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Teleport,
            target: Target::Enemy,
        });
        self
    }

    fn teleport_each(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Teleport,
            target: Target::Any,
        });
        self
    }

    fn call_assist(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::CallAssist,
            target: Target::Any,
        });
        self
    }

    fn call_assist_twice(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::CallAssistTwice,
            target: Target::Any,
        });
        self
    }

    fn backstab_any(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Backstab,
            target: Target::Any,
        });
        self
    }

    fn backstab_any_twice(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::BackstabTwice,
            target: Target::Any,
        });
        self
    }

    fn poison_any(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Poison,
            target: Target::Any,
        });
        self
    }

    fn rats(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Rats,
            target: Target::Any,
        });
        self
    }

    fn hypnosis_enemy(mut self) -> Self {
        self.actions.push(WrappedAction {
            action: Action::Hypnosis,
            target: Target::Enemy,
        });
        self
    }

    fn is_mandatory(mut self) -> Self {
        self.is_mandatory = true;
        self
    }
}

fn side(health: Health) -> FaceDef {
    FaceDef {
        health,
        allegiance: Allegiance::Hero,
        features: Features::NoFeature,
        reaction: None,
        reaction_assist: None,
        rows: vec![],
        assists: vec![],
        swarm: None,
        rage: 0,
        modifier: None,
    }
}

impl FaceDef {
    fn feature(mut self, features: Features) -> Self {
        self.features = self.features.union(features);
        self
    }

    fn energy(mut self) -> Self {
        self.features = self.features.union(Features::Energy);
        self
    }

    fn werewolf(mut self) -> Self {
        self.allegiance = Allegiance::Werewolf;
        self
    }

    fn reaction(mut self, reaction: Reaction) -> Self {
        self.reaction = Some(reaction);
        self
    }

    fn block_to_rotate(self) -> Self {
        self.reaction(Reaction::Standard(StandardReaction {
            trigger: ReactionTrigger::Block,
            outcome: Some(SelfAction::Rotate),
        }))
    }

    fn block_to_flip(self) -> Self {
        self.reaction(Reaction::Standard(StandardReaction {
            trigger: ReactionTrigger::Block,
            outcome: Some(SelfAction::Flip),
        }))
    }

    fn block_perm(self) -> Self {
        self.reaction(Reaction::Standard(StandardReaction {
            trigger: ReactionTrigger::Block,
            outcome: None,
        }))
    }

    fn dodge_to_rotate(self) -> Self {
        self.reaction(Reaction::Standard(StandardReaction {
            trigger: ReactionTrigger::Dodge,
            outcome: Some(SelfAction::Rotate),
        }))
    }

    fn call_assist_to_turn(self) -> Self {
        self.reaction(Reaction::Assist(RequestAssistReaction {
            outcome: Some(SelfAction::Rotate),
        }))
    }

    fn call_assist_perm(self) -> Self {
        self.reaction(Reaction::Assist(RequestAssistReaction { outcome: None }))
    }

    fn on_hit_reaction(self, actions: WhenHitType) -> Self {
        self.reaction(Reaction::WhenHit(actions))
    }

    fn provide_assist_block_rotate(mut self) -> Self {
        self.reaction_assist = Some(ProvideAssistReaction {
            trigger: ReactionTrigger::Block,
            assist_cost: SelfAction::Rotate,
        });
        self
    }

    fn provide_assist_block_flip(mut self) -> Self {
        self.reaction_assist = Some(ProvideAssistReaction {
            trigger: ReactionTrigger::Block,
            assist_cost: SelfAction::Flip,
        });
        self
    }

    fn provide_assist_dodge_rotate(mut self) -> Self {
        self.reaction_assist = Some(ProvideAssistReaction {
            trigger: ReactionTrigger::Dodge,
            assist_cost: SelfAction::Rotate,
        });
        self
    }

    fn provide_assist_dodge_flip(mut self) -> Self {
        self.reaction_assist = Some(ProvideAssistReaction {
            trigger: ReactionTrigger::Dodge,
            assist_cost: SelfAction::Flip,
        });
        self
    }

    fn roll(self) -> Self {
        self.reaction(Reaction::Roll)
    }

    fn add_row(mut self, row: Row) -> Self {
        self.rows.push(row);
        self
    }

    fn add_assist(mut self, row: Row) -> Self {
        self.assists.push(row);
        self
    }

    fn swarm(mut self, row: Row) -> Self {
        self.swarm = Some(row);
        self
    }

    fn rage(mut self, rage: ConditionCountType) -> Self {
        self.rage = rage;
        self
    }

    fn modifier_no_mandatory(mut self, amount: ModifierAmount) -> Self {
        self.modifier = Some(Modifier {
            amount,
            mandatory: None,
        });
        self
    }

    fn modifier_rotate(mut self, amount: ModifierAmount) -> Self {
        self.modifier = Some(Modifier {
            amount,
            mandatory: Some(SelfAction::Rotate),
        });
        self
    }
}

const CARDS_SIZE: usize = 128;
pub struct CardDefs {
    cards: [Option<CardDef>; CARDS_SIZE],
}

impl CardDefs {
    pub fn get_max_card_id(&self) -> CardId {
        let mut max_id: CardId = 0;
        for maybe_card in &self.cards {
            if let Some(card) = maybe_card {
                max_id = cmp::max(max_id, card.id);
            }
        }

        max_id
    }

    pub fn get_card_if_exists(&self, id: usize) -> Option<&CardDef> {
        if let Some(card) = &self.cards[id] {
            Some(&card)
        } else {
            None
        }
    }

    pub fn get_card(&self, id: usize) -> &CardDef {
        self.get_card_if_exists(id).unwrap()
    }

    pub fn get_cards_for_class(&self, class: Class) -> Vec<&CardDef> {
        let mut result = vec![];

        for maybe_card in &self.cards {
            if let Some(card) = maybe_card {
                if card.class == class {
                    result.push(card);
                }
            }
        }

        result
    }

    fn register_card(&mut self, card_def: CardDef) {
        let new_card_id = card_def.id as usize;
        if new_card_id >= CARDS_SIZE {
            panic!(
                "ID {} is too large. Increase the card_defs size",
                card_def.id
            )
        }
        let card_at_id = &self.cards[card_def.id as usize];
        if card_at_id.is_some() {
            panic!("Duplicate card id: {}", new_card_id)
        }
        self.cards[new_card_id] = Some(card_def);
    }

    pub fn new() -> CardDefs {
        const INIT: Option<CardDef> = None;
        let mut card_defs = CardDefs {
            cards: [INIT; CARDS_SIZE],
        };

        {
            let warrior = CharBuilder::new(Class::Warrior, Allegiance::Hero);
            card_defs.register_card(warrior.card(
                1,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                                .hit_any(2)
                                )
                        .add_row(row()
                                .delay_any(2)
                                .rotate()
                                ),
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                            .delay_any(1)
                            .hit_any(2)
                        )
                        .add_row(row()
                            .delay_any(2)
                        )
                        .block_to_rotate()
                        ,
                    FaceKey::C => side(Health::Empty)
                        .add_row(row()
                            .hit_any(2)
                        )
                        .add_row(row()
                            .delay_any(2)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                            .hit_any(4)
                        )
                        .add_row(row()
                            .quicken_any(2)
                        )
                        ,
                },
            ));

            card_defs.register_card(warrior.card(
                2,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                            .hit_any(2)
                            .rotate()
                        )
                        .add_row(row()
                            .delay_any(2)
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                            .heal_any()
                            .rotate()
                        )
                        .add_row(row()
                            .quicken_any(2)
                        )
                        .block_to_rotate()
                        ,
                    FaceKey::C => side(Health::Empty)
                        .add_row(row()
                            .quicken_any(2)
                        )
                        .add_row(row()
                            .delay_any(2)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                            .heal_any()
                            .rotate()
                        )
                        .add_row(row()
                            .delay_any(3)
                        )
                        ,
                },
            ));

            card_defs.register_card(warrior.card(
                3,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                            .quicken_any(2)
                            .rotate()
                        )
                        .add_row(row()
                            .delay_any(2)
                            .rotate()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                            .hit_any(4)
                            .hit_any(4)
                            .rotate()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .add_row(row()
                            .quicken_any(2)
                        )
                        .add_row(row()
                            .delay_any(2)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                            .hit_any(4)
                        )
                        .add_row(row()
                            .delay_any(1)
                        )
                        ,
                },
            ));

            card_defs.register_card(warrior.card(
                4,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                            .hit_any(4)
                        )
                        .add_row(row()
                            .quicken_any(2)
                        )
                        .block_to_rotate()
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                            .hit_any(2)
                        )
                        .add_row(row()
                            .delay_any(1)
                            .quicken_any(1)
                            .rotate()
                        )
                        ,
                    FaceKey::C => side(Health::Half)
                        .add_row(row()
                            .hit_any(4)
                        )
                        .add_row(row()
                            .quicken_any(1)
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .add_row(row()
                            .hit_any(2)
                        )
                        .add_row(row()
                            .heal_any()
                        )
                        ,
                },
            ));

            card_defs.register_card(warrior.card(
                5,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                            .quicken_any(2)
                            .hit_any(2)
                        )
                        .block_to_rotate()
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                            .hit_any(2)
                            .rotate()
                        )
                        .add_row(row()
                            .quicken_any(2)
                            .rotate()
                        )
                        ,
                    FaceKey::C => side(Health::Half)
                        .add_row(row()
                            .delay_any(3)
                            .hit_any(2)
                            .hit_any(2)
                            .rotate()
                        )
                        .add_row(row()
                            .quicken_any(2)
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .add_row(row()
                            .hit_any(4)
                        )
                        .add_row(row()
                            .quicken_any(1)
                        )
                        ,
                },
            ));
        }

        {
            let ogre = CharBuilder::new(Class::Ogre, Allegiance::Baddie);
            card_defs.register_card(ogre.card(
                6,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                            .pull_enemy(5)
                            .hit_enemy(1)
                            .rotate()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                            .hit_enemy(4)
                            .push_enemy(3)
                        )
                        .block_to_rotate()
                        ,
                    FaceKey::C => side(Health::Empty)
                        .add_row(row()
                            .hit_enemy(5)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .feature(Features::Weight)
                        .add_row(row()
                            .hit_enemy_inf()
                            .hit_enemy_inf()
                            .rotate()
                        )
                        .block_perm()
                        ,
                },
            ));

            card_defs.register_card(ogre.card(
                7,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .feature(Features::Weight)
                        .add_row(row()
                            .hit_enemy(4)
                            .rotate()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                            .heal_ally()
                            .rotate()
                        )
                        .add_row(row()
                            .hit_enemy(7)
                            .rotate()
                        )
                        .block_to_rotate()
                        ,
                    FaceKey::C => side(Health::Empty)
                        .add_row(row()
                            .hit_enemy(5)
                            .pull_ally(4)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                            .push_enemy(3)
                            .hit_enemy(5)
                        )
                        ,
                },
            ));

            card_defs.register_card(ogre.card(
                8,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                            .hit_enemy(4)
                            .rotate()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .feature(Features::Weight)
                        .add_row(row()
                            .heal_ally()
                            .rotate()
                        )
                        .add_row(row()
                            .hit_enemy(5)
                            .rotate()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .add_row(row()
                            .pull_enemy(6)
                            .hit_enemy(1)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                            .pull_enemy(4)
                            .hit_enemy(1)
                        )
                        ,
                },
            ));

            card_defs.register_card(ogre.card(
                9,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                            .push_enemy(3)
                            .push_enemy(3)
                            .rotate()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                            .hit_enemy(5)
                            .rotate()
                        )
                        .block_to_rotate()
                        ,
                    FaceKey::C => side(Health::Empty)
                        .feature(Features::Weight)
                        .add_row(row()
                            .hit_enemy_inf()
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                            .heal_ally()
                            .heal_ally()
                            .heal_ally()
                            .rotate()
                        )
                        .add_row(row()
                            .pull_ally(5)
                        )
                        ,
                },
            ));
        }

        {
            let huntress = CharBuilder::new(Class::Huntress, Allegiance::Hero);
            card_defs.register_card(huntress.card(
                10,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .feature(Features::Trap)
                        .add_row(row()
                                 .arrow_any()
                                 .rotate()
                                 )
                        .add_row(row()
                                .delay_ally(2)
                                .manouver()
                                )
                        ,
                    FaceKey::B => side(Health::Full)
                        .feature(Features::Trap)
                        .add_row(row()
                                .quicken_enemy(2)
                                .manouver()
                                .rotate()
                                )
                        .add_row(row()
                                 .heal_any()
                                )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .feature(Features::Trap)
                        .add_row(row()
                                 .manouver()
                                 )
                        .add_row(row()
                                 .hit_any(1)
                                 )
                        .add_row(row()
                                 .delay_any(1)
                                 )
                        ,
                    FaceKey::D => side(Health::Half)
                        .feature(Features::Trap)
                        .add_row(row()
                                 .double_arrow_any()
                                 )
                        .add_row(row()
                                 .manouver()
                                 )
                        .add_row(row()
                                 .quicken_any(1)
                                 )
                        ,
                },
            ));

            card_defs.register_card(huntress.card(
                11,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .dodge_to_rotate()
                        .add_row(row()
                                 .arrow_any()
                                 .rotate()
                                 )
                        .add_row(row()
                                .quicken_enemy(2)
                                .manouver()
                                .rotate()
                                )
                        .add_row(row()
                                .hit_any(1)
                                .rotate()
                                )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                                .delay_ally(2)
                                )
                        .add_row(row()
                                 .quicken_enemy(2)
                                )
                        .add_row(row()
                                 .manouver()
                                 .manouver()
                                )
                        ,
                    FaceKey::C => side(Health::Half)
                        .add_row(row()
                                 .hit_any(1)
                                 .hit_any(1)
                                 .rotate()
                                 )
                        .add_row(row()
                                 .quicken_enemy(2)
                                 .manouver()
                                 )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .add_row(row()
                                 .hit_any(1)
                                 .manouver()
                                 .delay_ally(2)
                                 )
                        ,
                },
            ));

            card_defs.register_card(huntress.card(
                12,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .dodge_to_rotate()
                        .add_row(row()
                                 .manouver()
                                 .quicken_enemy(2)
                                 .rotate()
                                 )
                        .add_row(row()
                                .delay_any(1)
                                .rotate()
                                )
                        ,
                    FaceKey::B => side(Health::Full)
                        .feature(Features::Trap)
                        .add_row(row()
                                .double_arrow_any()
                                )
                        .add_row(row()
                                 .delay_ally(2)
                                 .manouver()
                                )
                        ,
                    FaceKey::C => side(Health::Half)
                        .feature(Features::Trap)
                        .add_row(row()
                                 .delay_ally(2)
                                 .arrow_any()
                                 )
                        .add_row(row()
                                 .manouver()
                                 )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .feature(Features::Trap)
                        .add_row(row()
                                 .delay_ally(2)
                                 .quicken_enemy(2)
                                 )
                        .add_row(row()
                                 .manouver()
                                 .hit_any(1)
                                 )
                        ,
                },
            ));

            card_defs.register_card(huntress.card(
                13,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .feature(Features::Trap)
                        .add_row(row()
                                 .manouver()
                                 .rotate()
                                 )
                        .add_row(row()
                                .hit_any(1)
                                )
                        .add_row(row()
                                .delay_any(1)
                                )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                                .arrow_any()
                                .rotate()
                                )
                        .add_row(row()
                                 .manouver()
                                 .hit_any(1)
                                 .rotate()
                                )
                        .add_row(row()
                                 .quicken_enemy(2)
                                )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .feature(Features::Trap)
                        .add_row(row()
                                 .arrow_any()
                                 )
                        .add_row(row()
                                 .manouver()
                                 )
                        .add_row(row()
                                 .delay_any(1)
                                 )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                                 .arrow_any()
                                 )
                        .add_row(row()
                                 .quicken_enemy(2)
                                 .manouver()
                                 )
                        ,
                },
            ));

            card_defs.register_card(huntress.card(
                14,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .dodge_to_rotate()
                        .add_row(row()
                                 .arrow_any()
                                 .rotate()
                                 )
                        .add_row(row()
                                .quicken_enemy(2)
                                .rotate()
                                )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                                .double_arrow_any()
                                )
                        .add_row(row()
                                 .delay_ally(2)
                                 .manouver()
                                )
                        .add_row(row()
                                 .heal_any()
                                )
                        ,
                    FaceKey::C => side(Health::Half)
                        .feature(Features::Trap)
                        .add_row(row()
                                 .delay_ally(2)
                                 .quicken_enemy(2)
                                 )
                        .add_row(row()
                                 .manouver()
                                 .hit_any(1)
                                 )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .add_row(row()
                                 .arrow_any()
                                 )
                        .add_row(row()
                                 .delay_ally(2)
                                 )
                        .add_row(row()
                                 .heal_any()
                                 )
                        ,
                },
            ));
        }

        {
            let vampire = CharBuilder::new(Class::Vampire, Allegiance::Baddie);
            card_defs.register_card(vampire.card(
                15,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                            .hit_enemy(1)
                            .hit_enemy(1)
                            .flip()
                        )
                        .add_row(row()
                            .hit_enemy(6)
                            .rotate()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                            .pull_ally(3)
                            .pull_ally(3)
                        )
                        .add_row(row()
                            .hit_enemy(3)
                            .hit_enemy(3)
                            .hit_enemy(3)
                            .flip()
                        )
                        ,
                    FaceKey::C => side(Health::Half)
                        .add_row(row()
                            .hit_enemy(1)
                            .hit_enemy(1)
                            .rotate()
                        )
                        .add_row(row()
                            .hit_enemy(5)
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .add_row(row()
                            .push_enemy(3)
                        )
                        .add_row(row()
                            .hit_enemy(5)
                        )
                        ,
                },
            ));

            card_defs.register_card(vampire.card(
                16,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                            .inspire_ally()
                            .rotate()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                            .inspire_ally()
                            .inspire_ally()
                            .rotate()
                        )
                        ,
                    FaceKey::C => side(Health::Half)
                        .add_row(row()
                            .push_enemy(2)
                        )
                        .add_row(row()
                            .hit_enemy(5)
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .add_row(row()
                            .inspire_ally()
                        )
                        ,
                },
            ));

            card_defs.register_card(vampire.card(
                17,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                            .pull_enemy(3)
                            .hit_enemy(1)
                            .push_enemy(3)
                            .rotate()
                        )
                        .add_row(row()
                            .pull_ally(5)
                            .rotate()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                            .hit_enemy_inf()
                            .hit_enemy_inf()
                            .flip()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .add_row(row()
                            .hit_enemy(5)
                        )
                        .add_row(row()
                            .pull_ally(5)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                            .pull_ally(5)
                            .flip()
                        )
                        .add_row(row()
                            .pull_ally_inf()
                        )
                        ,
                },
            ));

            card_defs.register_card(vampire.card(
                18,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .feature(Features::Weight)
                        .add_row(row()
                            .add_condition(Condition::ExhaustedAllies(2))
                            .revive_ally()
                            .revive_ally()
                            .revive_ally()
                            .rotate()
                        )
                        .add_row(row()
                            .hit_enemy_inf()
                            .push_enemy(5)
                        )
                        .block_perm()
                        ,
                    FaceKey::B => side(Health::Full)
                        .feature(Features::Weight)
                        .add_row(row()
                            .add_condition(Condition::ExhaustedAllies(2))
                            .revive_ally()
                            .revive_ally()
                            .revive_ally()
                            .flip()
                        )
                        .add_row(row()
                            .push_enemy_inf()
                            .push_enemy_inf()
                        )
                        .block_perm()
                        ,
                    FaceKey::C => side(Health::Empty)
                        .add_row(row()
                            .push_enemy_inf()
                            .push_enemy_inf()
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .feature(Features::Weight)
                        .add_row(row()
                            .add_condition(Condition::ExhaustedAllies(2))
                            .revive_ally()
                            .revive_ally()
                            .revive_ally()
                            .rotate()
                        )
                        .add_row(row()
                            .hit_enemy_inf()
                        )
                        .block_perm()
                        ,
                },
            ));
        }

        {
            let pyro = CharBuilder::new(Class::Pyro, Allegiance::Hero);
            card_defs.register_card(pyro.card(
                19,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .energy()
                        .add_row(row()
                                 .energy_cost(2)
                                 .heal_any()
                                 .heal_any()
                                 .rotate()
                        )
                        .add_row(row()
                                 .energy_cost(1)
                                 .manouver()
                                 .manouver()
                                 .delay_any(2)
                                 .rotate()
                        )
                        .add_row(row()
                                 .delay_any(1)
                                 .quicken_any(1)
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .add_row(row()
                                 .energy_cost(3)
                                 .ablaze()
                                 .rotate()
                        )
                        .add_row(row()
                                 .energy_cost(2)
                                 .hit_any(3)
                                 .hit_any(3)
                        )
                        .add_row(row()
                                 .delay_any(1)
                                 .quicken_any(1)
                                 .rotate()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .add_row(row()
                                 .energy_cost(3)
                                 .teleport_enemy()
                                 .hit_enemy(2)
                        )
                        .add_row(row()
                                 .energy_cost(2)
                                 .teleport_each()
                                 .rotate()
                        )
                        .add_row(row()
                                 .teleport_ally()
                                 .rotate()
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .energy()
                        .add_row(row()
                                 .energy_cost(4)
                                 .revive_any()
                                 .flip()
                        )
                        .add_row(row()
                                 .manouver()
                                 .delay_any(2)
                        )
                        ,
                },
            ));

            card_defs.register_card(pyro.card(
                20,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                                 .energy_cost(2)
                                 .ablaze()
                        )
                        .add_row(row()
                                 .energy_cost(1)
                                 .hit_enemy(4)
                        )
                        .add_row(row()
                                 .heal_any()
                                 .manouver()
                                 .rotate()
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .energy()
                        .add_row(row()
                                 .energy_cost(2)
                                 .ablaze()
                                 .rotate()
                        )
                        .add_row(row()
                                 .energy_cost(1)
                                 .fireball()
                        )
                        .add_row(row()
                                 .delay_any(1)
                                 .quicken_any(1)
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .energy()
                        .add_row(row()
                                 .energy_cost(4)
                                 .hit_enemy(6)
                                 .hit_enemy(6)
                                 .hit_enemy(6)
                        )
                        .add_row(row()
                                 .energy_cost(2)
                                 .revive_any()
                                 .heal_any()
                                 .rotate()
                        )
                        .add_row(row()
                                 .delay_any(1)
                                 .quicken_any(1)
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .add_row(row()
                                 .energy_cost(2)
                                 .fireball()
                                 .rotate()
                        )
                        .add_row(row()
                                 .energy_cost(1)
                                 .hit_enemy(3)
                        )
                        .add_row(row()
                                 .heal_any()
                                 .manouver()
                        )
                        ,
                },
            ));

            card_defs.register_card(pyro.card(
                21,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                                 .energy_cost(2)
                                 .fireball()
                                 .rotate()
                        )
                        .add_row(row()
                                 .energy_cost(1)
                                 .heal_any()
                                 .heal_any()
                                 .rotate()
                        )
                        .add_row(row()
                                 .heal_any()
                                 .manouver()
                                 .rotate()
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .energy()
                        .add_row(row()
                                 .energy_cost(2)
                                 .revive_any()
                                 .rotate()
                        )
                        .add_row(row()
                                 .energy_cost(1)
                                 .delay_any(2)
                                 .quicken_any(2)
                        )
                        .add_row(row()
                                 .heal_any()
                                 .manouver()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .add_row(row()
                                 .energy_cost(3)
                                 .flip()
                        )
                        .add_row(row()
                                 .energy_cost(2)
                                 .teleport_each()
                        )
                        .add_row(row()
                                 .delay_any(2)
                                 .manouver()
                                 .rotate()
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .energy()
                        .add_row(row()
                                 .energy_cost(3)
                                 .revive_any()
                        )
                        .add_row(row()
                                 .energy_cost(2)
                                 .ablaze()
                        )
                        .add_row(row()
                                 .heal_any()
                                 .rotate()
                        )
                        ,
                },
            ));

            card_defs.register_card(pyro.card(
                22,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .energy()
                        .add_row(row()
                                 .energy_cost(2)
                                 .teleport_each()
                        )
                        .add_row(row()
                                 .energy_cost(1)
                                 .teleport_ally()
                        )
                        .add_row(row()
                                 .delay_any(1)
                                 .quicken_any(1)
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .add_row(row()
                                 .energy_cost(2)
                                 .hit_any(5)
                                 .heal_any()
                        )
                        .add_row(row()
                                 .energy_cost(1)
                                 .teleport_enemy()
                                 .rotate()
                        )
                        .add_row(row()
                                 .manouver()
                                 .heal_any()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .energy()
                        .add_row(row()
                                 .energy_cost(2)
                                 .fireball()
                                 .manouver()
                                 .rotate()
                        )
                        .add_row(row()
                                 .energy_cost(1)
                                 .hit_any_inf()
                                 .rotate()
                        )
                        .add_row(row()
                                 .manouver()
                                 .manouver()
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .add_row(row()
                                 .energy_cost(2)
                                 .ablaze()
                        )
                        .add_row(row()
                                 .energy_cost(1)
                                 .delay_any(2)
                                 .quicken_any(2)
                        )
                        .add_row(row()
                                 .delay_any(1)
                                 .quicken_any(1)
                                 .rotate()
                        )
                        ,
                },
            ));

            card_defs.register_card(pyro.card(
                23,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .energy()
                        .add_row(row()
                                 .energy_cost(3)
                                 .fireball()
                                 .fireball()
                        )
                        .add_row(row()
                                 .energy_cost(2)
                                 .ablaze()
                                 .heal_any()
                                 .flip()
                        )
                        .add_row(row()
                                 .delay_any(1)
                                 .manouver()
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .add_row(row()
                                 .energy_cost(2)
                                 .revive_any()
                                 .rotate()
                        )
                        .add_row(row()
                                 .energy_cost(2)
                                 .hit_any(5)
                                 .rotate()
                        )
                        .add_row(row()
                                 .teleport_ally()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .energy()
                        .add_row(row()
                                 .energy_cost(2)
                                 .ablaze()
                        )
                        .add_row(row()
                                 .energy_cost(1)
                                 .heal_any()
                                 .delay_any(1)
                                 .rotate()
                        )
                        .add_row(row()
                                 .quicken_any(1)
                                 .manouver()
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .add_row(row()
                                 .energy_cost(2)
                                 .hit_any(3)
                                 .flip()
                        )
                        .add_row(row()
                                 .teleport_ally()
                                 .rotate()
                        )
                        ,
                },
            ));
        }

        {
            let spider = CharBuilder::new(Class::Spider, Allegiance::Baddie);
            card_defs.register_card(spider.card(
                24,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .feature(Features::Venom)
                        .swarm(row()
                               .hit_enemy(3)
                               .pull_ally(3)
                               )
                        .add_row(row()
                            .hit_enemy(5)
                            .rotate()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .feature(Features::Venom)
                        .swarm(row()
                               .hit_enemy(2)
                               .push_enemy(3)
                               )
                        .add_row(row()
                            .push_enemy(3)
                            .hit_enemy(2)
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .feature(Features::Web)
                        .swarm(row()
                               .heal_ally()
                               )
                        .add_row(row()
                            .hit_enemy_inf()
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .feature(Features::Venom)
                        .swarm(row()
                               .pull_ally(4)
                               .push_enemy(3)
                               )
                        .add_row(row()
                            .hit_enemy(3)
                        )
                        .add_row(row()
                            .revive_ally()
                        )
                        ,
                },
            ));

            card_defs.register_card(spider.card(
                25,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .feature(Features::Venom)
                        .swarm(row()
                               .heal_ally()
                               .rotate()
                               )
                        .add_row(row()
                            .push_enemy(3)
                            .hit_enemy(3)
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .feature(Features::Venom)
                        .swarm(row()
                               .hit_enemy(3)
                               .push_enemy(2)
                               )
                        .add_row(row()
                            .hit_enemy(5)
                            .rotate()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .feature(Features::Venom)
                        .swarm(row()
                               .push_enemy(3)
                               .pull_ally(5)
                               )
                        .add_row(row()
                            .push_enemy(5)
                            .hit_enemy(2)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .feature(Features::Web)
                        .swarm(row()
                               .hit_enemy(5)
                               )
                        .add_row(row()
                            .hit_enemy(5)
                        )
                        ,
                },
            ));

            card_defs.register_card(spider.card(
                26,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .feature(Features::Venom)
                        .swarm(row()
                               .hit_enemy_inf()
                               )
                        .add_row(row()
                            .pull_ally(4)
                            .hit_enemy(2)
                            .rotate()
                        )
                        .add_row(row()
                            .hit_enemy(5)
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .feature(Features::Web)
                        .swarm(row()
                               .hit_enemy(4)
                               )
                        .add_row(row()
                            .push_enemy_inf()
                            .push_enemy_inf()
                            .push_ally_inf()
                            .rotate()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .feature(Features::Web)
                        .swarm(row()
                               .revive_ally()
                               )
                        .add_row(row()
                            .heal_ally()
                            .push_enemy(2)
                        )
                        .add_row(row()
                            .hit_enemy(4)
                            .push_enemy(4)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .feature(Features::Venom)
                        .swarm(row()
                               .hit_enemy(3)
                               )
                        .add_row(row()
                            .pull_ally(6)
                            .pull_ally(6)
                        )
                        ,
                },
            ));

            card_defs.register_card(spider.card(
                27,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .feature(Features::Web)
                        .swarm(row()
                               .hit_enemy(5)
                               )
                        .add_row(row()
                            .hit_enemy(3)
                            .rotate()
                        )
                        .add_row(row()
                            .push_enemy(3)
                            .push_enemy(3)
                        )
                        .add_row(row()
                            .revive_ally()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .feature(Features::Venom)
                        .block_to_rotate()
                        .swarm(row()
                               .hit_enemy(5)
                               .hit_enemy(5)
                               )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .feature(Features::Venom)
                        .swarm(row()
                               .hit_enemy_inf()
                               )
                        .add_row(row()
                            .hit_enemy(5)
                            .pull_ally(5)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .feature(Features::Web)
                        .swarm(row()
                               .hit_enemy_inf()
                               )
                        .add_row(row()
                            .hit_enemy(5)
                            .pull_ally(5)
                        )
                        ,
                },
            ));
        }

        {
            let werewolf = CharBuilder::new(Class::Cursed, Allegiance::Hero);
            card_defs.register_card(werewolf.card(
                28,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                                 .delay_ally(1)
                                 .heal_enemy()
                        )
                        .add_row(row()
                                 .manouver()
                                 .inspire_enemy()
                                 .manouver()
                        )
                        .add_row(row()
                                 .hit_ally_inf()
                                 .quicken_enemy(1)
                        )
                        ,
                    FaceKey::B => side(Health::Empty)
                        .add_row(row()
                                 .inspire_enemy()
                                 .heal_ally()
                        )
                        .add_row(row()
                                 .teleport_enemy()
                                 )
                        ,
                    FaceKey::C => side(Health::Half)
                        .werewolf()
                        .block_to_rotate()
                        .add_row(row()
                                 .claw_enemy(3)
                                 .rotate()
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                                 .delay_ally(2)
                                 .rotate()
                        )
                        .add_row(row()
                                 .quicken_enemy(2)
                        )
                        .add_row(row()
                                 .manouver()
                        )
                        ,
                },
            ));

            card_defs.register_card(werewolf.card(
                29,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                                 .hit_ally_inf()
                                 .quicken_enemy(1)
                        )
                        .add_row(row()
                                 .delay_any(1)
                                 .quicken_any(1)
                                 .manouver()
                        )
                        .add_row(row()
                                 .inspire_enemy()
                                 .quicken_any(2)
                        )
                        ,
                    FaceKey::B => side(Health::Empty)
                        .add_row(row()
                                 .manouver()
                                 .quicken_enemy(2)
                                 .inspire_enemy()
                        )
                        .add_row(row()
                                 .heal_any()
                        )
                        ,
                    FaceKey::C => side(Health::Half)
                        .werewolf()
                        .block_to_rotate()
                        .add_row(row()
                                 .claw_enemy(3)
                                 .rotate()
                        )
                        .add_row(row()
                                 .pull_enemy_inf()
                                 .hit_enemy(1)
                                 .hit_enemy(1)
                                 .hit_enemy(1)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                                .inspire_enemy()
                                .manouver()
                                .rotate()
                        )
                        .add_row(row()
                                .delay_ally(2)
                                .quicken_enemy(2)
                        )
                        ,
                },
            ));

            card_defs.register_card(werewolf.card(
                30,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                                 .quicken_enemy(2)
                                 .quicken_enemy(2)
                                 .inspire_enemy()
                        )
                        .add_row(row()
                                 .delay_ally(1)
                                 .hit_ally_inf()
                        )
                        ,
                    FaceKey::B => side(Health::Empty)
                        .add_row(row()
                                 .manouver()
                                 .quicken_enemy(2)
                                 .inspire_enemy()
                        )
                        .add_row(row()
                                 .heal_any()
                                 .inspire_enemy()
                        )
                        ,
                    FaceKey::C => side(Health::Half)
                        .werewolf()
                        .block_to_rotate()
                        .add_row(row()
                                 .hit_enemy(2)
                                 .hit_enemy(2)
                                 .rotate()
                        )
                        .add_row(row()
                                 .claw_enemy(4)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                                 .manouver()
                                 .heal_any()
                                 .rotate()
                        )
                        .add_row(row()
                                 .delay_ally(1)
                                 .quicken_enemy(1)
                        )
                        .add_row(row()
                                 .teleport_each()
                        )
                        ,
                },
            ));

            card_defs.register_card(werewolf.card(
                31,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                                 .heal_enemy()
                                 .manouver()
                        )
                        .add_row(row()
                                 .quicken_enemy(2)
                                 .delay_ally(2)
                                 .inspire_enemy()
                        )
                        .add_row(row()
                                 .hit_ally_inf()
                                 .quicken_enemy(1)
                        )
                        ,
                    FaceKey::B => side(Health::Empty)
                        .add_row(row()
                                 .teleport_enemy()
                                 .inspire_enemy()
                        )
                        .add_row(row()
                                 .teleport_each()
                                 .manouver()
                        )
                        ,
                    FaceKey::C => side(Health::Half)
                        .werewolf()
                        .block_to_rotate()
                        .add_row(row()
                                 .hit_enemy(2)
                                 .hit_enemy(2)
                                 .push_enemy(2)
                                 .rotate()
                        )
                        .add_row(row()
                                 .hit_enemy_inf()
                                 .hit_enemy_inf()
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                                 .heal_enemy()
                                 .rotate()
                        )
                        .add_row(row()
                                 .teleport_each()
                                 .manouver()
                        )
                        .add_row(row()
                                 .hit_ally_inf()
                                 .delay_enemy(1)
                        )
                        ,
                },
            ));

            card_defs.register_card(werewolf.card(
                32,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                                 .teleport_enemy()
                                 .inspire_enemy()
                        )
                        .add_row(row()
                                 .manouver()
                                 .heal_enemy()
                                 .flip()
                        )
                        .add_row(row()
                                 .hit_ally_inf()
                                 .delay_enemy(1)
                        )
                        ,
                    FaceKey::B => side(Health::Empty)
                        .add_row(row()
                                 .hit_ally_inf()
                                 .inspire_enemy()
                                 .manouver()
                        )
                        .add_row(row()
                                 .quicken_any(1)
                                 .delay_any(1)
                        )
                        ,
                    FaceKey::C => side(Health::Half)
                        .werewolf()
                        .block_to_rotate()
                        .add_row(row()
                                 .pull_enemy(6)
                                 .hit_enemy(3)
                                 .hit_enemy(3)
                                 .hit_enemy(3)
                                 .rotate()
                        )
                        .add_row(row()
                                 .hit_enemy_inf()
                                 )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                                 .inspire_enemy()
                                 .inspire_enemy()
                                 .delay_any(1)
                        )
                        .add_row(row()
                                 .teleport_enemy()
                                 .manouver()
                        )
                        .add_row(row()
                                 .heal_enemy()
                                 .rotate()
                        )
                        ,
                },
            ));
        }

        {
            let demon = CharBuilder::new(Class::Demon, Allegiance::Baddie);
            card_defs.register_card(demon.card(
                33,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                                .rage_condition(3)
                                .pull_enemy_inf()
                                .hit_enemy(1)
                                .hit_enemy(1)
                        )
                        .add_row(row()
                                .rage_condition(2)
                                .pull_enemy_inf()
                                .hit_enemy(1)
                        )
                        .add_row(row()
                                .push_enemy(3)
                                .push_enemy(3)
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .rage(2)
                        .add_row(row()
                                .rage_condition(3)
                                .pull_ally(5)
                                .rotate()
                        )
                        .add_row(row()
                                .rage_condition(2)
                                .pull_ally(5)
                                .push_enemy(5)
                        )
                        .add_row(row()
                                .hit_enemy(3)
                                .push_enemy(3)
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .rage(2)
                        .block_to_rotate()
                        .add_row(row()
                                .rage_condition(4)
                                .void()
                        )
                        .add_row(row()
                                .rage_condition(2)
                                .hit_enemy(1)
                                .hit_enemy(2)
                                .hit_enemy(3)
                        )
                        .add_row(row()
                                .hit_enemy(2)
                                .hit_enemy(2)
                                .rotate()
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .rage(1)
                        .block_to_rotate()
                        .add_row(row()
                                .rage_condition(4)
                                .death()
                        )
                        .add_row(row()
                                .rage_condition(3)
                                .claw_enemy(4)
                                .rotate()
                        )
                        .add_row(row()
                                .pull_ally(5)
                                .rotate()
                        )
                        ,
                },
            ));

            card_defs.register_card(demon.card(
                34,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                                .rage_condition(5)
                                .death()
                        )
                        .add_row(row()
                                .rage_condition(3)
                                .void()
                        )
                        .add_row(row()
                                .hit_enemy_inf()
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .rage(1)
                        .add_row(row()
                                .rage_condition(4)
                                .void()
                                .rotate()
                        )
                        .add_row(row()
                                .rage_condition(2)
                                .hit_enemy_inf()
                        )
                        .add_row(row()
                                .claw_enemy(2)
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .rage(2)
                        .block_to_rotate()
                        .add_row(row()
                                .rage_condition(4)
                                .death()
                        )
                        .add_row(row()
                                .rage_condition(3)
                                .pull_enemy(5)
                                .hit_enemy(1)
                                .hit_enemy(2)
                                .push_enemy(1)
                        )
                        .add_row(row()
                                .pull_enemy(5)
                                .hit_enemy(1)
                                .hit_enemy(1)
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .rage(1)
                        .block_to_rotate()
                        .add_row(row()
                                .rage_condition(4)
                                .death()
                        )
                        .add_row(row()
                                .rage_condition(2)
                                .hit_enemy_inf()
                                .push_enemy_inf()
                                .rotate()
                        )
                        .add_row(row()
                                .hit_enemy_inf()
                                .rotate()
                        )
                        ,
                },
            ));

            card_defs.register_card(demon.card(
                35,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                                .rage_condition(5)
                                .claw_enemy_inf()
                        )
                        .add_row(row()
                                .rage_condition(2)
                                .claw_enemy(4)
                        )
                        .add_row(row()
                                .claw_enemy(2)
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .rage(2)
                        .add_row(row()
                                .rage_condition(4)
                                .claw_enemy(6)
                                .rotate()
                        )
                        .add_row(row()
                                .rage_condition(3)
                                .claw_enemy(4)
                                .rotate()
                        )
                        .add_row(row()
                                .hit_enemy_inf()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .rage(2)
                        .block_to_rotate()
                        .add_row(row()
                                .rage_condition(4)
                                .death()
                        )
                        .add_row(row()
                                .claw_enemy(3)
                                .rotate()
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .rage(1)
                        .block_to_rotate()
                        .add_row(row()
                                .rage_condition(5)
                                .death()
                        )
                        .add_row(row()
                                .rage_condition(3)
                                .hit_enemy_inf()
                                .hit_enemy_inf()
                                .rotate()
                        )
                        .add_row(row()
                                .hit_enemy_inf()
                                .rotate()
                        )
                        ,
                },
            ));

            card_defs.register_card(demon.card(
                36,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .add_row(row()
                                .rage_condition(3)
                                .void()
                        )
                        .add_row(row()
                                .rage_condition(2)
                                .hit_enemy(4)
                                .hit_enemy(4)
                        )
                        .add_row(row()
                                .hit_enemy_inf()
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .rage(1)
                        .add_row(row()
                                .rage_condition(4)
                                .hit_enemy_inf()
                                .hit_enemy_inf()
                                .rotate()
                        )
                        .add_row(row()
                                .rage_condition(3)
                                .hit_enemy_inf()
                                .rotate()
                        )
                        .add_row(row()
                                .hit_enemy(4)
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .rage(2)
                        .block_to_rotate()
                        .add_row(row()
                                .rage_condition(5)
                                .death()
                        )
                        .add_row(row()
                                .rage_condition(3)
                                .void()
                        )
                        .add_row(row()
                                .pull_ally(5)
                                .rotate()
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .rage(1)
                        .block_to_rotate()
                        .add_row(row()
                                .rage_condition(4)
                                .void()
                        )
                        .add_row(row()
                                .rage_condition(3)
                                .pull_enemy(5)
                                .hit_enemy(1)
                                .pull_enemy(5)
                                .hit_enemy(1)
                                .rotate()
                        )
                        .add_row(row()
                                .pull_enemy(5)
                                .hit_enemy(1)
                                .rotate()
                        )
                        ,
                },
            ));
        }

        {
            let beastmaster = CharBuilder::new(Class::Beastmaster, Allegiance::Hero);
            card_defs.register_card(beastmaster.card(
                37,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .call_assist_to_turn()
                        .add_row(row()
                                .call_assist_twice()
                                .rotate()
                        )
                        .add_row(row()
                                .teleport_enemy()
                                .call_assist()
                        )
                        .add_row(row()
                                .teleport_ally()
                                .call_assist()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                                .call_assist_twice()
                                .rotate()
                        )
                        .add_row(row()
                                .hit_any(2)
                                .call_assist()
                        )
                        .add_row(row()
                                .manouver()
                                .arrow_any()
                        )
                        ,
                    FaceKey::C => side(Health::Half)
                        .call_assist_perm()
                        .add_row(row()
                                .call_assist()
                                .hit_any(2)
                        )
                        .add_row(row()
                                .quicken_any(2)
                                .call_assist()
                        )
                        .add_row(row()
                                .manouver()
                                .call_assist()
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .add_row(row()
                                .hit_any(2)
                                .call_assist()
                        )
                        .add_row(row()
                                .call_assist_twice()
                        )
                        .add_row(row()
                                .manouver()
                                .manouver()
                        )
                        ,
                },
            ));

            card_defs.register_card(beastmaster.card(
                38,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .call_assist_to_turn()
                        .add_row(row()
                                .call_assist_twice()
                                .rotate()
                        )
                        .add_row(row()
                                .call_assist()
                                .hit_any(2)
                        )
                        .add_row(row()
                                .delay_any(2)
                                .call_assist()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                                .call_assist()
                                .quicken_any(2)
                        )
                        .add_row(row()
                                .hit_any(2)
                                .call_assist()
                        )
                        .add_row(row()
                                .manouver()
                                .rotate()
                        )
                        ,
                    FaceKey::C => side(Health::Half)
                        .call_assist_perm()
                        .add_row(row()
                                .call_assist()
                                .hit_any(1)
                        )
                        .add_row(row()
                                .call_assist_twice()
                        )
                        .add_row(row()
                                .teleport_ally()
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .add_row(row()
                                .quicken_any(1)
                                .delay_any(1)
                        )
                        .add_row(row()
                                .call_assist_twice()
                        )
                        .add_row(row()
                                .teleport_enemy()
                                .call_assist()
                        )
                        ,
                },
            ));

            card_defs.register_card(beastmaster.card(
                39,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .call_assist_to_turn()
                        .add_row(row()
                                .hit_any(2)
                                .call_assist()
                                .rotate()
                        )
                        .add_row(row()
                                .teleport_enemy()
                                .call_assist()
                        )
                        .add_row(row()
                                .call_assist()
                                .manouver()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                                .manouver()
                                .teleport_ally()
                                .rotate()
                        )
                        .add_row(row()
                                .teleport_enemy()
                                .call_assist()
                        )
                        .add_row(row()
                                .call_assist_twice()
                        )
                        ,
                    FaceKey::C => side(Health::Half)
                        .call_assist_perm()
                        .add_row(row()
                                .call_assist()
                                .manouver()
                        )
                        .add_row(row()
                                .arrow_any()
                                .call_assist()
                        )
                        .add_row(row()
                                .call_assist()
                                .teleport_ally()
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .add_row(row()
                                .quicken_enemy(2)
                                .call_assist()
                                .delay_enemy(2)
                        )
                        .add_row(row()
                                .call_assist_twice()
                        )
                        .add_row(row()
                                .teleport_enemy()
                                .call_assist()
                        )
                        ,
                },
            ));

            card_defs.register_card(beastmaster.card(
                40,
                enum_map! {
                    FaceKey::A => side(Health::Empty)
                        .provide_assist_block_rotate()
                        .add_assist(row()
                                .claw_enemy(2)
                                .rotate()
                        )
                        .add_assist(row()
                                .hit_any(1)
                        )
                        .add_assist(row()
                                .quicken_ally(2)
                        )
                        .add_row(row()
                                 .quicken_ally(2)
                                 .flip()
                                 )
                        ,
                    FaceKey::B => side(Health::Empty)
                        .add_assist(row()
                                .hit_any(1)
                        )
                        .add_assist(row()
                                .quicken_ally(1)
                        )
                        .add_row(row()
                                 .rotate()
                                 )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .provide_assist_block_flip()
                        .add_assist(row()
                                .claw_enemy(3)
                                .flip()
                        )
                        .add_assist(row()
                                .hit_any(2)
                        )
                        .add_assist(row()
                                .teleport_ally()
                        )
                        .add_row(row()
                                 .quicken_ally(3)
                                 .rotate()
                                 )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .provide_assist_block_rotate()
                        .add_assist(row()
                                .claw_enemy(4)
                                .rotate()
                        )
                        .add_assist(row()
                                .teleport_each()
                                .rotate()
                        )
                        .add_assist(row()
                                .hit_any(3)
                        )
                        .add_row(row()
                                 .teleport_each()
                        )
                        ,
                },
            ));

            card_defs.register_card(beastmaster.card(
                41,
                enum_map! {
                    FaceKey::A => side(Health::Empty)
                        .provide_assist_dodge_rotate()
                        .add_assist(row()
                                .arrow_any()
                                .rotate()
                        )
                        .add_assist(row()
                                .delay_enemy(2)
                        )
                        .add_row(row()
                                 .delay_enemy(2)
                                 .flip()
                                 )
                        ,
                    FaceKey::B => side(Health::Empty)
                        .add_assist(row()
                                .delay_enemy(1)
                        )
                        .add_row(row()
                                 .rotate()
                                 )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .provide_assist_dodge_flip()
                        .add_assist(row()
                                .arrow_any()
                                .flip()
                        )
                        .add_assist(row()
                                .heal_any()
                                .flip()
                        )
                        .add_assist(row()
                                .teleport_enemy()
                        )
                        .add_row(row()
                                 .delay_enemy(3)
                                 .rotate()
                                 )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .provide_assist_dodge_rotate()
                        .feature(Features::Trap)
                        .add_assist(row()
                                .arrow_any()
                                .rotate()
                        )
                        .add_assist(row()
                                .revive_any()
                                .flip()
                        )
                        .add_assist(row()
                                .teleport_enemy()
                        )
                        .add_row(row()
                                 .delay_enemy(2)
                                 .quicken_ally(2)
                        )
                        ,
                },
            ));
        }

        {
            let verdant = CharBuilder::new(Class::Flora, Allegiance::Baddie);
            card_defs.register_card(verdant.card(
                42,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .on_hit_reaction(&ROW_PUSH_ENEMY_INF_PUSH_ENEMY_INF)
                        .add_row(row()
                                 .hit_ally(2)
                                 .pull_ally(6)
                                 )
                        .add_row(row()
                                 .hit_ally_inf()
                                 .heal_ally()
                                 )
                        ,
                    FaceKey::B => side(Health::Empty)
                        .add_row(row()
                                 .heal_ally()
                                 )
                        .add_row(row()
                                 .hit_ally(3)
                                 .pull_ally(5)
                                 )
                        .add_row(row()
                                 .revive_ally()
                                 )
                        ,
                    FaceKey::C => side(Health::Half)
                        .feature(Features::Weight)
                        .on_hit_reaction(&ROW_PUSH_ALLY_INF_PULL_ALLY_INF)
                        .add_row(row()
                                 .heal_ally()
                                 )
                        .add_row(row()
                                 .push_enemy(3)
                                 .hit_enemy(3)
                                 .push_enemy(2)
                                 )
                        ,
                    FaceKey::D => side(Health::Half)
                        .feature(Features::Weight)
                        .on_hit_reaction(&ROW_CLAW_ODD_ENEMIES)
                        .add_row(row()
                                 .push_enemy(2)
                                 .push_enemy(3)
                                 )
                        ,
                },
            ));

            card_defs.register_card(verdant.card(
                43,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .on_hit_reaction(&ROW_CLAW_ENEMY_4)
                        .add_row(row()
                                 .claw_enemy(2)
                                 )
                        .add_row(row()
                                 .hit_enemy(4)
                                 )
                        ,
                    FaceKey::B => side(Health::Empty)
                        .add_row(row()
                                 .hit_enemy(3)
                                 )
                        .add_row(row()
                                 .heal_ally()
                                 )
                        .add_row(row()
                                 .hit_ally_inf()
                                 .heal_ally()
                                 )
                        ,
                    FaceKey::C => side(Health::Half)
                        .feature(Features::Weight)
                        .on_hit_reaction(&ROW_CLAW_EVEN_ENEMIES)
                        .add_row(row()
                                 .inspire_ally()
                                 )
                        ,
                    FaceKey::D => side(Health::Half)
                        .feature(Features::Weight)
                        .on_hit_reaction(&ROW_CLAW_ODD_ENEMIES)
                        .add_row(row()
                                 .hit_enemy(3)
                                 .push_enemy(2)
                                 )
                        ,
                },
            ));

            card_defs.register_card(verdant.card(
                44,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .on_hit_reaction(&ROW_CLAW_ENEMY_4)
                        .add_row(row()
                                 .hit_enemy(4)
                                 )
                        ,
                    FaceKey::B => side(Health::Empty)
                        .add_row(row()
                                 .hit_enemy(3)
                                 )
                        .add_row(row()
                                 .heal_ally()
                                 )
                        .add_row(row()
                                 .hit_ally_inf()
                                 .heal_ally()
                                 )
                        ,
                    FaceKey::C => side(Health::Half)
                        .feature(Features::Weight)
                        .on_hit_reaction(&ROW_CLAW_ODD_ENEMIES)
                        .add_row(row()
                                 .hit_enemy(3)
                                 )
                        ,
                    FaceKey::D => side(Health::Half)
                        .feature(Features::Weight)
                        .on_hit_reaction(&ROW_CLAW_EVEN_ENEMIES)
                        .add_row(row()
                                 .hit_enemy(3)
                                 )
                        ,
                },
            ));

            card_defs.register_card(verdant.card(
                45,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .on_hit_reaction(&ROW_HEAL_ALLY)
                        .add_row(row()
                                 .hit_enemy(2)
                                 .push_enemy(2)
                                 )
                        .add_row(row()
                                 .revive_ally()
                                 )
                        .add_row(row()
                                 .hit_ally(4)
                                 .pull_ally(5)
                                 )
                        ,
                    FaceKey::B => side(Health::Empty)
                        .add_row(row()
                                 .heal_ally()
                                 .hit_enemy(3)
                                 )
                        .add_row(row()
                                 .hit_enemy(6)
                                 )
                        ,
                    FaceKey::C => side(Health::Half)
                        .feature(Features::Weight)
                        .on_hit_reaction(&ROW_REVIVE_ALLY)
                        .add_row(row()
                                 .push_enemy(2)
                                 )
                        ,
                    FaceKey::D => side(Health::Half)
                        .feature(Features::Weight)
                        .on_hit_reaction(&ROW_HEAL_ALLY_CLAW_ENEMY_3)
                        .add_row(row()
                                 .push_enemy(2)
                                 .push_enemy(2)
                                 )
                        ,
                },
            ));
        }

        {
            let assassin = CharBuilder::new(Class::Assassin, Allegiance::Hero);
            card_defs.register_card(assassin.card(
                46,
                enum_map! {
                    FaceKey::A => side(Health::Half)
                        .dodge_to_rotate()
                        .add_row(row()
                                .dodge_cost(3)
                                .backstab_any_twice()
                                .rotate()
                        )
                        .add_row(row()
                                .dodge_cost(1)
                                .quicken_ally(2)
                        )
                        .add_row(row()
                                .manouver()
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .add_row(row()
                                .dodge_cost(2)
                                .backstab_any()
                        )
                        .add_row(row()
                                .dodge_cost(1)
                                .poison_any()
                        )
                        .add_row(row()
                                .delay_ally(2)
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .roll()
                        .add_row(row()
                                .dodge_cost(2)
                                .poison_any()
                        )
                        .add_row(row()
                                .dodge_cost(2)
                                .delay_ally(1)
                                .manouver()
                        )
                        .add_row(row()
                                .quicken_ally(1)
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .dodge_to_rotate()
                        .add_row(row()
                                .dodge_cost(3)
                                .flip()
                        )
                        .add_row(row()
                                .dodge_cost(2)
                                .backstab_any()
                        )
                        .add_row(row()
                                .manouver()
                                .manouver()
                        )
                        ,
                },
            ));

            card_defs.register_card(assassin.card(
                47,
                enum_map! {
                    FaceKey::A => side(Health::Half)
                        .dodge_to_rotate()
                        .add_row(row()
                                .dodge_cost(2)
                                .backstab_any()
                        )
                        .add_row(row()
                                .dodge_cost(1)
                                .poison_any()
                        )
                        .add_row(row()
                                .delay_ally(1)
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .roll()
                        .add_row(row()
                                .dodge_cost(2)
                                .poison_any()
                                .poison_any()
                        )
                        .add_row(row()
                                .dodge_cost(2)
                                .backstab_any()
                        )
                        .add_row(row()
                                .delay_ally(1)
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .dodge_to_rotate()
                        .add_row(row()
                                .dodge_cost(4)
                                .revive_any()
                                .rotate()
                        )
                        .add_row(row()
                                .dodge_cost(3)
                                .backstab_any_twice()
                        )
                        .add_row(row()
                                .delay_ally(1)
                                .delay_enemy(1)
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .add_row(row()
                                .dodge_cost(2)
                                .backstab_any()
                        )
                        .add_row(row()
                                .dodge_cost(1)
                                .poison_any()
                        )
                        .add_row(row()
                                .manouver()
                                .delay_ally(1)
                        )
                        ,
                },
            ));

            card_defs.register_card(assassin.card(
                48,
                enum_map! {
                    FaceKey::A => side(Health::Half)
                        .dodge_to_rotate()
                        .add_row(row()
                                .dodge_cost(2)
                                .backstab_any()
                        )
                        .add_row(row()
                                .dodge_cost(2)
                                .poison_any()
                        )
                        .add_row(row()
                                .delay_enemy(1)
                                .manouver()
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .add_row(row()
                                .dodge_cost(3)
                                .backstab_any()
                        )
                        .add_row(row()
                                .dodge_cost(1)
                                .poison_any()
                        )
                        .add_row(row()
                                .delay_ally(1)
                                .delay_enemy(1)
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .dodge_to_rotate()
                        .add_row(row()
                                .dodge_cost(3)
                                .teleport_ally()
                                .teleport_enemy()
                        )
                        .add_row(row()
                                .dodge_cost(2)
                                .poison_any()
                        )
                        .add_row(row()
                                .quicken_enemy(1)
                                .delay_enemy(1)
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .roll()
                        .add_row(row()
                                .dodge_cost(4)
                                .revive_any()
                        )
                        .add_row(row()
                                .dodge_cost(2)
                                .backstab_any()
                                .rotate()
                        )
                        .add_row(row()
                                .manouver()
                                .delay_enemy(1)
                        )
                        ,
                },
            ));

            card_defs.register_card(assassin.card(
                49,
                enum_map! {
                    FaceKey::A => side(Health::Half)
                        .roll()
                        .add_row(row()
                                .dodge_cost(2)
                                .backstab_any()
                        )
                        .add_row(row()
                                .dodge_cost(1)
                                .quicken_ally(2)
                                .delay_enemy(2)
                        )
                        .add_row(row()
                                .manouver()
                                .manouver()
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .dodge_to_rotate()
                        .add_row(row()
                                .dodge_cost(3)
                                .delay_ally(2)
                                .manouver()
                                .quicken_ally(2)
                        )
                        .add_row(row()
                                .dodge_cost(1)
                                .poison_any()
                                .rotate()
                        )
                        .add_row(row()
                                .delay_enemy(1)
                                .manouver()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .add_row(row()
                                .dodge_cost(2)
                                .poison_any()
                                .rotate()
                        )
                        .add_row(row()
                                .dodge_cost(1)
                                .delay_enemy(2)
                        )
                        .add_row(row()
                                 .manouver()
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .dodge_to_rotate()
                        .add_row(row()
                                .dodge_cost(4)
                                .revive_any()
                                .rotate()
                        )
                        .add_row(row()
                                .dodge_cost(1)
                                .backstab_any()
                                .rotate()
                        )
                        .add_row(row()
                                .delay_ally(1)
                        )
                        ,
                },
            ));

            card_defs.register_card(assassin.card(
                50,
                enum_map! {
                    FaceKey::A => side(Health::Half)
                        .dodge_to_rotate()
                        .add_row(row()
                                .dodge_cost(2)
                                .backstab_any()
                        )
                        .add_row(row()
                                .dodge_cost(2)
                                .poison_any()
                        )
                        .add_row(row()
                                .delay_enemy(1)
                                .manouver()
                        )
                        ,
                    FaceKey::B => side(Health::Half)
                        .add_row(row()
                                .dodge_cost(3)
                                .poison_any()
                                .poison_any()
                        )
                        .add_row(row()
                                .dodge_cost(2)
                                .backstab_any()
                                .rotate()
                        )
                        .add_row(row()
                                .quicken_enemy(1)
                                .delay_ally(1)
                                .rotate()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .roll()
                        .add_row(row()
                                .dodge_cost(3)
                                .flip()
                        )
                        .add_row(row()
                                .dodge_cost(1)
                                .quicken_ally(2)
                        )
                        .add_row(row()
                                 .manouver()
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .dodge_to_rotate()
                        .add_row(row()
                                .dodge_cost(2)
                                .backstab_any()
                        )
                        .add_row(row()
                                .dodge_cost(2)
                                .poison_any()
                        )
                        .add_row(row()
                                .manouver()
                        )
                        ,
                },
            ));
        }

        {
            let wall = CharBuilder::new(Class::Wall, Allegiance::Baddie);
            card_defs.register_card(wall.back_card(
                51,
                enum_map! {
                    FaceKey::A => side(Health::Empty)
                        .feature(Features::Wall)
                        .feature(Features::Invulnerable)
                        .add_row(row()
                                .pull_ally_inf()
                                .rotate()
                        )
                        ,
                    FaceKey::B => side(Health::Empty)
                        .feature(Features::Wall)
                        .feature(Features::Invulnerable)
                        .block_perm()
                        .add_row(row()
                                .heal_ally()
                                .flip()
                                .is_mandatory()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .feature(Features::Wall)
                        .feature(Features::Invulnerable)
                        .add_row(row()
                                .death()
                        )
                        ,
                    FaceKey::D => side(Health::Empty)
                        .feature(Features::Wall)
                        .feature(Features::Invulnerable)
                        .block_perm()
                        .add_row(row()
                                .push_enemy_inf()
                                .pull_ally_inf()
                                .rotate()
                        )
                        ,
                },
            ));

            card_defs.register_card(wall.card(
                52,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .feature(Features::Weight)
                        .block_to_rotate()
                        ,
                    FaceKey::B => side(Health::Full)
                        .block_to_flip()
                        .add_row(row()
                                .rotate()
                                .is_mandatory()
                        )
                        ,
                    FaceKey::C => side(Health::Empty),
                    FaceKey::D => side(Health::Half)
                        .feature(Features::Weight)
                        .block_to_rotate()
                        .add_row(row()
                                .flip()
                                .is_mandatory()
                        )
                        ,
                },
            ));

            card_defs.register_card(wall.card(
                53,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .feature(Features::Weight)
                        .block_to_rotate()
                        ,
                    FaceKey::B => side(Health::Full)
                        .block_to_flip()
                        ,
                    FaceKey::C => side(Health::Empty)
                        .add_row(row()
                                .rotate()
                                .is_mandatory()
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .feature(Features::Weight)
                        .block_to_rotate()
                        .add_row(row()
                                .flip()
                                .is_mandatory()
                        )
                        ,
                },
            ));

            card_defs.register_card(wall.card(
                54,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .block_to_rotate()
                        ,
                    FaceKey::B => side(Health::Full)
                        .feature(Features::Weight)
                        .block_to_flip()
                        .add_row(row()
                                .rotate()
                                .is_mandatory()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .add_row(row()
                                .rotate()
                                .is_mandatory()
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .block_to_rotate()
                        ,
                },
            ));
        }

        {
            let piper = CharBuilder::new(Class::Piper, Allegiance::Hero);
            card_defs.register_card(piper.card(
                55,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .modifier_rotate(-2)
                        .add_row(row()
                                .arrow_any()
                                .quicken_enemy(2)
                        )
                        .add_row(row()
                                .teleport_enemy()
                                .manouver()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .modifier_rotate(1)
                        .add_row(row()
                                .hypnosis_enemy()
                                .rotate()
                        )
                        .add_row(row()
                                .hit_any(0)
                                .rotate()
                        )
                        .add_row(row()
                                .delay_enemy(2)
                                .quicken_ally(2)
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .modifier_no_mandatory(1)
                        .add_row(row()
                                .hypnosis_enemy()
                        )
                        .add_row(row()
                                .rats()
                        )
                        .add_row(row()
                                .teleport_enemy()
                                .manouver()
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .modifier_rotate(4)
                        .add_row(row()
                                .hypnosis_enemy()
                        )
                        .add_row(row()
                                .arrow_any()
                                .teleport_enemy()
                        )
                        .add_row(row()
                                .delay_enemy(2)
                                .manouver()
                        )
                        ,
                },
            ));

            card_defs.register_card(piper.card(
                56,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .modifier_rotate(-2)
                        .add_row(row()
                                .hypnosis_enemy()
                                .rotate()
                        )
                        .add_row(row()
                                .rats()
                                .teleport_enemy()
                        )
                        .add_row(row()
                                .manouver()
                                .quicken_any(1)
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .modifier_rotate(1)
                        .add_row(row()
                                .arrow_any()
                                .delay_ally(1)
                        )
                        .add_row(row()
                                .hit_any(0)
                                .quicken_any(1)
                        )
                        .add_row(row()
                                .manouver()
                                .manouver()
                                .rotate()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .modifier_no_mandatory(-1)
                        .add_row(row()
                                .arrow_any()
                                .quicken_enemy(1)
                        )
                        .add_row(row()
                                .teleport_enemy()
                                .manouver()
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .modifier_rotate(-4)
                        .add_row(row()
                                .hypnosis_enemy()
                        )
                        .add_row(row()
                                .rats()
                        )
                        .add_row(row()
                                .teleport_each()
                        )
                        ,
                },
            ));

            card_defs.register_card(piper.card(
                57,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .modifier_rotate(1)
                        .add_row(row()
                                .hit_any(0)
                                .quicken_enemy(1)
                        )
                        .add_row(row()
                                .teleport_each()
                                .rotate()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .modifier_rotate(-1)
                        .add_row(row()
                                .arrow_any()
                                .teleport_enemy()
                        )
                        .add_row(row()
                                .quicken_ally(1)
                                .quicken_ally(1)
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .modifier_no_mandatory(1)
                        .add_row(row()
                                .arrow_any()
                                .quicken_enemy(1)
                        )
                        .add_row(row()
                                .hit_any(0)
                                .manouver()
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .modifier_rotate(4)
                        .add_row(row()
                                .hypnosis_enemy()
                        )
                        .add_row(row()
                                .rats()
                        )
                        .add_row(row()
                                .hit_any(0)
                                .teleport_each()
                        )
                        ,
                },
            ));

            card_defs.register_card(piper.card(
                58,
                enum_map! {
                    FaceKey::A => side(Health::Full)
                        .modifier_rotate(1)
                        .add_row(row()
                                .hypnosis_enemy()
                                .quicken_enemy(2)
                        )
                        .add_row(row()
                                .rats()
                                .delay_ally(2)
                        )
                        .add_row(row()
                                .quicken_enemy(1)
                                .quicken_enemy(1)
                                .manouver()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .modifier_rotate(-1)
                        .add_row(row()
                                .teleport_enemy()
                                .arrow_any()
                                .rotate()
                        )
                        .add_row(row()
                                .delay_ally(1)
                                .delay_enemy(1)
                                .manouver()
                        )
                        ,
                    FaceKey::C => side(Health::Empty)
                        .modifier_no_mandatory(-1)
                        .add_row(row()
                                .hypnosis_enemy()
                        )
                        .add_row(row()
                                .rats()
                        )
                        .add_row(row()
                                .hit_any(0)
                                .delay_enemy(1)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .modifier_rotate(-4)
                        .add_row(row()
                                .arrow_any()
                                .teleport_enemy()
                        )
                        .add_row(row()
                                .quicken_enemy(2)
                                .manouver()
                        )
                        ,
                },
            ));

            let rat = CharBuilder::new(Class::Piper, Allegiance::Rat);
            card_defs.register_card(rat.card(
                59,
                enum_map! {
                    FaceKey::A => side(Health::Empty)
                        .add_row(row()
                                .claw_enemy(0)
                                .is_mandatory()
                        )
                        ,
                    FaceKey::B => side(Health::Full)
                        .add_row(row()
                                 .death()
                        )
                        ,
                    FaceKey::C => side(Health::Half)
                        .add_row(row()
                                .claw_enemy(2)
                        )
                        ,
                    FaceKey::D => side(Health::Half)
                        .add_row(row()
                                .hit_enemy(2)
                                .hit_enemy(2)
                        )
                        ,
                },
            ));
        }

        /*
            {
                let wisp = CharBuilder::new(Class::Wisp, Allegiance::Baddie);
                card_defs.register_card(wisp.card(
                    55,
                    enum_map! {
                        FaceKey::A => side(Health::Full)
                            .feature(Features::Wisp)
                            .add_row(row()
                                .pull_enemy(5)
                                .hit_enemy(1)
                                .rotate()
                            )
                            ,
                        FaceKey::B => side(Health::Full)
                            .feature(Features::Wisp)
                            .add_row(row()
                                .hit_enemy(4)
                                .push_enemy(3)
                            )
                            .block_to_rotate()
                            ,
                        FaceKey::C => side(Health::Empty)
                            .feature(Features::Wisp)
                            .add_row(row()
                                .hit_enemy(5)
                            )
                            ,
                        FaceKey::D => side(Health::Half)
                            .feature(Features::Wisp)
                            .feature(Features::Weight)
                            .add_row(row()
                                .hit_enemy_inf()
                                .hit_enemy_inf()
                                .rotate()
                            )
                            .block_perm()
                            ,
                    },
                ));

                card_defs.register_card(wisp.card(
                    56,
                    enum_map! {
                        FaceKey::A => side(Health::Full)
                            .feature(Features::Wisp)
                            .feature(Features::Weight)
                            .add_row(row()
                                .hit_enemy(4)
                                .rotate()
                            )
                            ,
                        FaceKey::B => side(Health::Full)
                            .feature(Features::Wisp)
                            .add_row(row()
                                .heal_ally()
                                .rotate()
                            )
                            .add_row(row()
                                .hit_enemy(7)
                                .rotate()
                            )
                            .block_to_rotate()
                            ,
                        FaceKey::C => side(Health::Empty)
                            .feature(Features::Wisp)
                            .add_row(row()
                                .hit_enemy(5)
                                .pull_ally(4)
                            )
                            ,
                        FaceKey::D => side(Health::Half)
                            .feature(Features::Wisp)
                            .add_row(row()
                                .push_enemy(3)
                                .hit_enemy(5)
                            )
                            ,
                    },
                ));

                card_defs.register_card(wisp.card(
                    57,
                    enum_map! {
                        FaceKey::A => side(Health::Full)
                            .feature(Features::Wisp)
                            .add_row(row()
                                .hit_enemy(4)
                                .rotate()
                            )
                            ,
                        FaceKey::B => side(Health::Full)
                            .feature(Features::Wisp)
                            .feature(Features::Weight)
                            .add_row(row()
                                .heal_ally()
                                .rotate()
                            )
                            .add_row(row()
                                .hit_enemy(5)
                                .rotate()
                            )
                            ,
                        FaceKey::C => side(Health::Empty)
                            .feature(Features::Wisp)
                            .add_row(row()
                                .pull_enemy(6)
                                .hit_enemy(1)
                            )
                            ,
                        FaceKey::D => side(Health::Half)
                            .feature(Features::Wisp)
                            .add_row(row()
                                .pull_enemy(4)
                                .hit_enemy(1)
                            )
                            ,
                    },
                ));

                card_defs.register_card(wisp.card(
                    58,
                    enum_map! {
                        FaceKey::A => side(Health::Full)
                            .feature(Features::Wisp)
                            .add_row(row()
                                .push_enemy(3)
                                .push_enemy(3)
                                .rotate()
                            )
                            ,
                        FaceKey::B => side(Health::Full)
                            .feature(Features::Wisp)
                            .add_row(row()
                                .hit_enemy(5)
                                .rotate()
                            )
                            .block_to_rotate()
                            ,
                        FaceKey::C => side(Health::Empty)
                            .feature(Features::Wisp)
                            .feature(Features::Weight)
                            .add_row(row()
                                .hit_enemy_inf()
                            )
                            ,
                        FaceKey::D => side(Health::Half)
                            .feature(Features::Wisp)
                            .add_row(row()
                                .heal_ally()
                                .heal_ally()
                                .heal_ally()
                                .rotate()
                            )
                            .add_row(row()
                                .pull_ally(5)
                            )
                            ,
                    },
                ));
            }
        */

        card_defs
    }
}
