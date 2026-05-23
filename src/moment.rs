// zephyr repl cipher html writer
// no depends no splits
//
// file writes to '/output/html'
//  read a file from '/input'
//
//
//

use std::collections::HashMap;
use std::fs;
// use std::io::{self, BufRead, Write};
use std::path::Path;
// use std::time::{SystemTime, UNIX_EPOCH};

// #define-esque
const FILE_READ_STOP: u64 = 1_048_576; // 1024*1024 as ~1MB and 1MiB

// maintained across commands
pub struct Moment {
    pub text: Option<String>, // working plaintext\ciphertext
    pub outdir: String,       // html files directory
    pub log: Vec<Logger>,     // all files written in session
}

// one entry per html file -- enough to rebuild index.html
pub struct Logger {
    pub namefiles: String, // write_015.html
    pub setcipher: String, // mode used
    pub summarize: String, // params used
    pub previewer: String, // print 'x' chars to cons?
}

impl Moment {
    fn new() -> Self {
        Moment {
            text: None,
            outdir: "output/html".to_string(),
            log: Vec::new(),
        }
    }
}

// ..Flags
// how commands are parsed
//  csr /s 20 /d
//  vig /k word
//  set /t thisisadefaultmessage
//
// split on whitespace? collect "/key word" pairs into HashMap
// first word is command always... check below???

struct Flags {
    command: String,
    map: HashMap<String, String>,
    // bare words collected before and /flag appear
    // "set attack at dawn" => position = ["attack", "at", "dawn"]
    position: Vec<String>,
}

impl Flags {
    fn parser(liner: &str) -> Self {
        let tokens: Vec<&str> = liner.split_whitespace().collect();
        let command = tokens.first().unwrap_or(&"").to_lowercase();

        let mut map = HashMap::new();
        let mut position = Vec::new();
        let mut i = 1;
        while i < tokens.len() {
            if let Some(key) = tokens[i].strip_prefix("/") {
                if i + 1 < tokens.len() && !tokens[i + 1].starts_with("/") {
                    map.insert(key.to_lowercase(), tokens[i + 1].to_string());
                    i += 1;
                } else {
                    map.insert(key.to_lowercase(), String::new());
                    i += 1;
                }
            } else {
                position.push(tokens[i].to_string());
                i += 1; //
            }
        }
        Flags { command, map, position, }
    }

    // all position words joined with spaces
    // avoids having to use shell quotes
    // on input text for whole text strings
    // set this phrase here =>
    // position_text() = "this phrase here"
    fn check_position(&self) -> String {
        self.position.join(" ")
    }

    // get a string flag or fallback
    fn read_get<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.map.get(key).map(|s| s.as_str()).unwrap_or(default)
    }

    // check if a flag was preset at all (on /d)
    fn flag_off(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    // get integer flag or fallback
    fn read_num(&self, key: &str, default: i32) -> i32 {
        self.read_get(key, "").parse().unwrap_or(default)
    }

    // parse /r /range 1..50 or fall back to /from /to or other defaults
    fn read_range(&self) -> (usize, usize) {
        if let Some(r) = self.map.get("range") {
            if let Some((a, b)) = r.split_once("..") {
                let from = a.parse().unwrap_or(1);
                let to: usize = b.parse().unwrap_or(999);
                return (from, to.max(from));
            }
        }
        let from = self.read_get("from", "1").parse().unwrap_or(1);
        let to = self.read_get("to", "999").parse().unwrap_or(999);
        (from, to.max(from))
    }
}

fn file_read(path: &str) -> Result<String, String> {
    // try /input/<path> first, then the path as given
    let resolver = {
        let form_input = Path::new("input").join(path);
        if form_input.exists() {
            form_input
        } else {
            Path::new(path).to_path_buf()
        }
    };
    // guard here checks the file size before 'taking' a single byte
    let meta =
        fs::metadata(&resolver).map_err(|_| format!("null*{} check/{path}", resolver.display()))?;
    if meta.len() > FILE_READ_STOP {
        return Err(format!(
            "{} of {:.1}KB is too large, limit is {}",
            resolver.display(),
            meta.len() as f64 / 1024.0,
            FILE_READ_STOP / 1024
        ));
    }

    fs::read_to_string(&resolver)
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("null*read {}: {e}", resolver.display()))
}

fn get_txt(f: &Flags, moment: &Moment) -> Result<String, String> {
    // 1.) /text flag (single word only, flag parser limitation)
    let t = f.read_get("text", "");
    if !t.is_empty() {
        return Ok(t.to_string());
    }
    let t = f.read_get("t", "");
    if !t.is_empty() {
        return Ok(t.to_string());
    }
    // 2.) /file flag ... read from disk via 'file_read()'
    // whick checks /input/ first and enforces size limit
    let path = f.read_get("file", "");
    if !path.is_empty() {
        return file_read(path);
    }
    let path = f.read_get("f", "");
    if !path.is_empty() {
        return file_read(path);
    }
    // 3.) position of word on same line (many word no quotes)
    //  ex. "csr this is a string of words /s 12"
    //      uses "this is a string of words"
    let pos = f.check_position();
    if !pos.is_empty() {
        return Ok(pos);
    }
    // 4.) fallback to whatever was stored by "set ..."
    moment
        .text
        .clone()
        .ok_or_else(|| "try: set --text message".to_string())
}
