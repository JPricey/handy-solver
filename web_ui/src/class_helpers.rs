use handy_core::game::*;

pub fn class_to_character_name(class: Class) -> &'static str {
    match class {
        Class::Dummy => "Dummy",
        Class::Warrior => "Ragnar",
        Class::Huntress => "Kisah",
        Class::Pyro => "Leo",
        Class::Cursed => "Jiro",
        Class::Beastmaster => "Zora",
        Class::Assassin => "Assassin",
        Class::Monk => "Monk",
        Class::Ogre => "Gonk",
        Class::Vampire => "Marius",
        Class::Spider => "Arach",
        Class::Demon => "Dargoth",
        Class::Flora => "Verdancy",
        Class::Wall => "Wall",
        Class::Piper => "Piper",
        Class::Troupe => "Troupe",
        Class::Ooze => "Ooze",
    }
}

pub fn class_to_full_class_name(class: Class) -> &'static str {
    match class {
        Class::Dummy => "Dummy",
        Class::Warrior => "Warrior",
        Class::Huntress => "Huntress",
        Class::Pyro => "Pyromancer",
        Class::Cursed => "Cursed",
        Class::Beastmaster => "Beastmaster",
        Class::Assassin => "Assassin (BETA)",
        Class::Monk => "Monk (BETA)",
        Class::Ogre => "Ogre",
        Class::Vampire => "Vampire",
        Class::Spider => "Spider Swarm",
        Class::Demon => "Blood Demon",
        Class::Flora => "Sentient Flora",
        Class::Wall => "Wall",
        Class::Piper => "Pied Piper",
        Class::Troupe => "Circus Troupe",
        Class::Ooze => "Ooze",
    }
}
