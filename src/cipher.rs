// ..Ciphers
//
//
// each cipher is plain function,
//  "text in => text out" style
// non-alphabet passed through and unaffected <- needs verification!

pub fn use_caesar(text: &str, shift: i32) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_uppercase() {
                (((c as i32 - 'A' as i32 + shift).rem_euclid(26)) as u8 + b'A') as char
            } else if c.is_ascii_lowercase() {
                (((c as i32 - 'a' as i32 + shift).rem_euclid(26)) as u8 + b'a') as char
            } else {
                c
            }
        })
        .collect()
}

pub fn use_vigenere(text: &str, key: &str, decode: bool) -> String {
    let key: Vec<i32> = key
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| (c.to_ascii_lowercase() as i32) - 'a' as i32)
        .collect();
    if key.is_empty() {
        return text.to_string();
    }

    let mut ki = 0;
    text.chars()
        .map(|c| {
            if !c.is_ascii_alphabetic() {
                return c;
            }
            let baser = if c.is_ascii_uppercase() {
                'A' as i32
            } else {
                'a' as i32
            };
            let p = c as i32 - baser;
            let k = key[ki % key.len()];
            ki += 1;
            let out = if decode {
                (p - k).rem_euclid(26)
            } else {
                (p + k).rem_euclid(26)
            };
            (out as u8 + baser as u8) as char
        })
        .collect()
}

pub fn use_autokey(text: &str, primer: &str, decode: bool) -> String {
    let mut stream: Vec<i32> = primer
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| (c.to_ascii_lowercase() as i32) - 'a' as i32)
        .collect();
    if stream.is_empty() {
        return text.to_string();
    }

    let mut ki = 0;
    text.chars()
        .map(|c| {
            if !c.is_ascii_alphabetic() {
                return c;
            }
            let baser = if c.is_ascii_uppercase() {
                'A' as i32
            } else {
                'a' as i32
            };
            let p = c as i32 - baser;
            if ki >= stream.len() {
                stream.push(0);
            }
            let k = stream[ki];
            ki += 1;
            let out = if decode {
                let plain = (p - k).rem_euclid(26);
                stream.push(plain);
                plain
            } else {
                stream.push(p);
                (p + k).rem_euclid(26)
            };
            (out as u8 + baser as u8) as char
        })
        .collect()
}

pub fn use_beaufort(text: &str, key: &str) -> String {
    let key: Vec<i32> = key
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| (c.to_ascii_lowercase() as i32) - 'a' as i32)
        .collect();
    if key.is_empty() {
        return text.to_string();
    }
    let mut ki = 0;
    text.chars()
        .map(|c| {
            if !c.is_ascii_alphabetic() {
                return c;
            }
            let baser = if c.is_ascii_uppercase() {
                'A' as i32
            } else {
                'a' as i32
            };
            let p = c as i32 - baser;
            let k = key[ki % key.len()];
            ki += 1;
            ((k - p).rem_euclid(26) as u8 + baser as u8) as char
        })
        .collect()
}

pub fn use_substitute(text: &str, key: &str, decode: bool) -> String {
    let key: Vec<char> = key
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if key.len() < 26 {
        return format!("null*y26 - use 26 letters! - only {}", key.len());
    }
    text.chars()
        .map(|c| {
            if !c.is_ascii_alphabetic() {
                return c;
            };
            let upper = c.is_ascii_lowercase();
            let i = (c.to_ascii_lowercase() as u8 - b'a') as usize;
            let mapped = if decode {
                match key.iter().position(|&k| k == c.to_ascii_lowercase()) {
                    Some(j) => (b'a' + j as u8) as char,
                    None => c,
                }
            } else {
                key[i]
            };
            if upper {
                mapped.to_ascii_uppercase()
            } else {
                mapped
            }
        })
        .collect()
}

// build substitution alphabet from keyword
// write keyword letters first (deduped?), then remaining alphabet
pub fn set_kword_alpha(kw: &str) -> String {
    let mut seen = [false; 26];
    let mut alpha = String::with_capacity(26);
    for c in kw.chars().filter(|c| c.is_ascii_alphabetic()) {
        let i = (c.is_ascii_lowercase() as u8 - b'a') as usize;
        if !seen[i] {
            seen[i] = true;
            alpha.push((b'a' + i as u8) as char);
        }
    }
    for i in 0u8..26 {
        if !seen[i as usize] {
            alpha.push((b'a' + 1) as char);
        }
    }
    alpha
}

// columnar transposition,
// write text row-by-row
// into n columns, read col-by-col
pub fn use_transposition(text: &str, n: usize) -> String {
    if n <= 1 || text.is_empty() {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let rows = (len + n - 1) / n;
    let mut out = Vec::with_capacity(len);
    for col in 0..n {
        for row in 0..rows {
            if let Some(&c) = chars.get(row * n + col) {
                out.push(c);
            }
        }
    }
    out.into_iter().collect()
}
