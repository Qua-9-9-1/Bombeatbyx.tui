pub const ALL_SKINS: &[&str] = &["👶", "😑", "😎", "🤓", "🥰", "😱", "🤠", "🥸", "👿", "🐷", "🐸", "🎃", "🗿", "😡", "💩", "🤡", "👹", "👺", "👽", "🤖", "🍔", "🍙", "🍪", "🍎", "🥔"];

pub fn get_skin_name(skin: &str) -> &'static str {
    match skin {
        "👶" => "Baby",
        "😑" => "Neutral",
        "😎" => "Cool",
        "🤓" => "Nerd",
        "🥰" => "Romantic",
        "😱" => "Scared",
        "🤠" => "Cowboy",
        "🥸" => "Disguised",
        "👿" => "Devil",
        "🐷" => "Pig",
        "🐸" => "Frog",
        "🎃" => "Jack-o'-Lantern",
        "🗿" => "Golem",
        "😡" => "Angry",
        "💩" => "Poop",
        "🤡" => "Clown",
        "👹" => "Ogre",
        "👺" => "Goblin",
        "👽" => "Alien",
        "🤖" => "Robot",
        "🍔" => "Burger",
        "🍙" => "Rice",
        "🍪" => "Cookie",
        "🍎" => "Apple",
        "🥔" => "Potato",
        _ => "Player",
    }
}

pub fn get_skin_short_code(skin: &str) -> &'static str {
    match skin {
        "👶" => "BB",
        "😑" => "NT",
        "😎" => "CL",
        "🤓" => "ND",
        "🥰" => "RM",
        "😱" => "SC",
        "🤠" => "CW",
        "🥸" => "DS",
        "👿" => "DV",
        "🐷" => "PG",
        "🐸" => "FR",
        "🎃" => "JK",
        "🗿" => "GL",
        "😡" => "AN",
        "💩" => "PT",
        "🤡" => "CL",
        "👹" => "OG",
        "👺" => "GB",
        "👽" => "AL",
        "🤖" => "RO",
        "🍔" => "BG",
        "🍙" => "RC",
        "🍪" => "CK",
        "🍎" => "AP",
        "🥔" => "PT",
        _ => "PL",
    }
}

pub fn is_valid_player_name(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() || name.chars().count() > 16 {
        return false;
    }
    name.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_player_name() {
        assert!(is_valid_player_name("Player1"));
        assert!(is_valid_player_name("Jean-Michel"));
        assert!(is_valid_player_name("Aôut_éè"));
        assert!(is_valid_player_name("Bot_1"));
        
        assert!(!is_valid_player_name(""));
        assert!(!is_valid_player_name("   "));
        assert!(!is_valid_player_name("PlayerNameIsTooLongToBeValidForThisGame"));
        assert!(!is_valid_player_name("Player#1"));
        assert!(!is_valid_player_name("🤖"));
    }

}
