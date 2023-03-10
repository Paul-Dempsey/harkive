use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

pub fn count_leading(leading: char, slice: &str) -> u32 {
    let mut count = 0;
    for item in slice.chars() {
        if item != leading {
            break;
        }
        count += 1;
    }
    count
}

pub fn make_hex_string(data: &[u8]) -> String {
    let mut hex_data = String::default();
    let mut count = 0;
    for byte in data.iter() {
        hex_data += &format!("{:02X}", byte);
        count += 1;
        if count == 4 {
            hex_data.push(' ');
            count = 0;
        }
    }
    hex_data
}

pub fn display_f64(f: f64) -> String {
    let raw = format!("{f:.3}");
    let result = raw.trim_end_matches(&['0', '.']);
    if result.is_empty() {
        return "0".to_string();
    } else if result.len() == 1 {
        if let Some(ch) = result.chars().next() {
            if ch == '-' {
                return "0".to_string();
            }
        }
    }
    result.to_string()
}

pub fn maybe_char(ch: u8) -> String {
    if (32..127).contains(&ch) {
        format!(" #{:02X} {}", ch, ch as char)
    } else {
        format!(" #{ch:02X}")
    }
}

pub fn on_off(b: bool) -> &'static str {
    match b {
        true => "on",
        false => "off",
    }
}
pub fn yes_no(b: bool) -> &'static str {
    match b {
        true => "yes",
        false => "no",
    }
}
pub fn preserve_replace(b: bool) -> &'static str {
    match b {
        true => "Preserve",
        false => "Replace",
    }
}

pub fn fill_box(b: bool) -> char {
    match b {
        true => '◆',
        false => '◇',
    }
}

#[inline]
pub fn hi_nybble(byte: u8) -> u8 {
    byte & 0xF0
}
#[inline]
pub fn lo_nybble(byte: u8) -> u8 {
    byte & 0x0F
}
#[inline]
pub fn hi_word(value: u64) -> u32 {
    ((value & 0xFFFF0000) >> 32) as u32
}
#[inline]
pub fn lo_word(value: u64) -> u32 {
    (value & 0x0000FFFF) as u32
}
#[inline]
pub fn xor_fold_u64(value: u64) -> u32 {
    hi_word(value) ^ lo_word(value)
}

pub fn short_hash(data: &[u8]) -> u32 {
    let mut hasher = DefaultHasher::new();
    hasher.write(data);
    xor_fold_u64(hasher.finish())
}

// pub fn is_bit(value: u8, bit: u8) -> bool {
//     assert!((0..8).contains(&bit));
//     0 != value & (1 << (bit))
// }

#[inline]
/// lowest bit (1) set (0x01)
pub fn is_bit1(value: u8) -> bool {
    0 != value & 0x01
}
#[inline]
/// bit 2 set (0x02)
pub fn is_bit2(value: u8) -> bool {
    0 != value & 0x02
}
#[inline]
/// bit 3 set (0x04)
pub fn is_bit3(value: u8) -> bool {
    0 != value & 0x04
}
#[inline]
/// bit 4 set (0x08)
pub fn is_bit4(value: u8) -> bool {
    0 != value & 0x08
}
#[inline]
/// bit 5 set (0x10)
pub fn is_bit5(value: u8) -> bool {
    0 != value & 0x10
}
#[inline]
/// bit 6 set (0x20)
pub fn is_bit6(value: u8) -> bool {
    0 != value & 0x20
}
#[inline]
/// bit 7 set (0x40)
pub fn is_bit7(value: u8) -> bool {
    0 != value & 0x40
}
#[inline]
/// bit 8 set (0x80)
pub fn is_bit8(value: u8) -> bool {
    0 != value & 0x80
}

pub fn get_u16(data: &[u8]) -> u16 {
    u16::from_be_bytes(data[..2].try_into().expect("u16"))
}

pub fn u16_from_midi_bytes(lo: u8, hi: u8) -> u16 {
    debug_assert!(!is_bit8(lo));
    ((hi as u16) << 7) | (lo as u16)
}

pub fn get_u32(data: &[u8]) -> u32 {
    u32::from_be_bytes(data[..4].try_into().expect("u32"))
}

/// Brazenly borrowed from
/// https://github.com/TheAlgorithms/Rust/blob/master/src/dynamic_programming/edit_distance.rs
///
/// Instead of storing the `m * n` matrix expicitly, only one row (of length `n`) is stored.
/// It keeps overwriting itself based on its previous values with the help of two scalars,
/// gradually reaching the last row. Then, the score is `matrix[n]`.
///
/// # Complexity
///
/// - time complexity: O(nm),
/// - space complexity: O(n),
///
/// where n and m are lengths of `str_a` and `str_b`
///
pub fn edit_distance(str_a: &str, str_b: &str) -> u32 {
    let (str_a, str_b) = (str_a.as_bytes(), str_b.as_bytes());
    let (m, n) = (str_a.len(), str_b.len());
    let mut distances: Vec<u32> = vec![0; n + 1]; // the dynamic programming matrix (only 1 row stored)
    let mut s: u32; // distances[i - 1][j - 1] or distances[i - 1][j]
    let mut c: u32; // distances[i][j - 1] or distances[i][j]
    let mut char_a: u8; // str_a[i - 1] the i-th character in str_a; only needs to be computed once per row
    let mut char_b: u8; // str_b[j - 1] the j-th character in str_b

    // 0th row
    for (j, v) in distances.iter_mut().enumerate().take(n + 1).skip(1) {
        *v = j as u32;
    }
    // rows 1 to m
    for i in 1..=m {
        s = (i - 1) as u32;
        c = i as u32;
        char_a = str_a[i - 1];
        for j in 1..=n {
            // c is distances[i][j-1] and s is distances[i-1][j-1] at the beginning of each round of iteration
            char_b = str_b[j - 1];
            c = std::cmp::min(
                s + u32::from(char_a != char_b),
                std::cmp::min(c + 1, distances[j] + 1),
            );
            // c is updated to distances[i][j], and will thus become distances[i][j-1] for the next cell
            s = distances[j]; // here distances[j] means distances[i-1][j] because it has not been overwritten yet
                              // s is updated to distances[i-1][j], and will thus become distances[i-1][j-1] for the next cell
            distances[j] = c; // now distances[j] is updated to distances[i][j], and will thus become distances[i-1][j] for the next ROW
        }
    }

    distances[n]
}
