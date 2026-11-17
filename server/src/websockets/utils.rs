use crate::state::Room;

use std::hash::{BuildHasher, Hasher};

/// Unpredictable 64-bit value: `RandomState` keys come from the OS random source.
fn random_u64() -> u64 {
    std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish()
}

/// Private room codes act as passwords, so they must not be guessable
/// from the creation time.
pub fn generate_room_code(rooms: &std::collections::HashMap<String, Room>) -> String {
    let chars: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    loop {
        let mut code = String::new();
        for _ in 0..6 {
            let idx = (random_u64() % chars.len() as u64) as usize;
            code.push(chars[idx] as char);
        }
        if !rooms.contains_key(&code) {
            return code;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn generate_room_code_formatting_and_collision_avoidance() {
        let mut rooms = HashMap::new();
        
        let code = generate_room_code(&rooms);

        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric() && (c.is_uppercase() || c.is_numeric())));

        rooms.insert(code.clone(), Room::new(code.clone(), true, false));

        let code_2 = generate_room_code(&rooms);

        assert_ne!(code, code_2);
        assert_eq!(code_2.len(), 6);
    }
}
