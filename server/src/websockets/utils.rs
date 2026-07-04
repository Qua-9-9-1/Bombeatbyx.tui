use crate::state::Room;

pub fn generate_room_code(rooms: &std::collections::HashMap<String, Room>) -> String {
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let chars: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    loop {
        let mut code = String::new();
        for _ in 0..6 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let idx = (seed >> 32) as usize % chars.len();
            code.push(chars[idx] as char);
        }
        if !rooms.contains_key(&code) {
            return code;
        }
    }
}
