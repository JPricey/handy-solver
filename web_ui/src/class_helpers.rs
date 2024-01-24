use handy_core::game::*;

pub fn class_to_character_name(class: Class) -> &'static str {
    match class {
        Class::Warrior => "Ragnar",
        Class::Huntress => "Kisah",
        Class::Pyro => "Leo",
        Class::Cursed => "Jiro",
        Class::Beastmaster => "Zora",
        Class::Assassin => "Assassin",
        Class::Ogre => "Gonk",
        Class::Vampire => "Marius",
        Class::Spider => "Arach",
        Class::Demon => "Dargoth",
        Class::Flora => "Verdancy",
        Class::Wall => "Wall",
    }
}


pub fn class_to_full_class_name(class: Class) -> &'static str {
    match class {
        Class::Warrior => "Warrior",
        Class::Huntress => "Huntress",
        Class::Pyro => "Pyromancer",
        Class::Cursed => "Cursed",
        Class::Beastmaster => "Beastmaster",
        Class::Assassin => "Assassin (BETA)",
        Class::Ogre => "Ogre",
        Class::Vampire => "Vampire",
        Class::Spider => "Spider Swarm",
        Class::Demon => "Blood Demon",
        Class::Flora => "Sentient Flora",
        Class::Wall => "Wall (BETA)",
    }
}
