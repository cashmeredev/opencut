use std::cell::Cell;
use std::time::{SystemTime, UNIX_EPOCH};

thread_local! {
    static RNG_STATE: Cell<u64> = const { Cell::new(0) };
}

fn next_u64() -> u64 {
    RNG_STATE.with(|state| {
        let mut value = state.get();
        if value == 0 {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos() as u64)
                .unwrap_or(0x9e37_79b9_7f4a_7c15);
            value = nanos
                ^ (std::process::id() as u64).rotate_left(32)
                ^ 0x9e37_79b9_7f4a_7c15;
        }
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        state.set(value);
        value
    })
}

pub fn generate_uuid() -> String {
    let mut bytes = [0u8; 16];
    for chunk in bytes.chunks_mut(8) {
        let random = next_u64().to_le_bytes();
        chunk.copy_from_slice(&random[..chunk.len()]);
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex: Vec<String> = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        hex[0..4].join(""),
        hex[4..6].join(""),
        hex[6..8].join(""),
        hex[8..10].join(""),
        hex[10..16].join("")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_uuid_shaped_unique_ids() {
        let id = generate_uuid();
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[4].len(), 12);
        assert_ne!(generate_uuid(), generate_uuid());
    }
}
