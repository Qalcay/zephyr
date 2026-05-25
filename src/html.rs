//use std::fs;
//use std::path::Path;
use crate::moment::*;
use crate::cipher::*;

// ..HTML
//
// two pages,
//  result page -   where the ciphered output goes
//  index page  -   lists every resulting file produced
//
// both written to moment.outdir ("/output/html")
//
// shared css formatting defined here once, as a constant and inlined into all pages
// that way each file is self contained, you can open it on its own

pub const CSS: &str = r#"
<style>
@import url('https://fonts.googleapis.com/css2?family=Google+Sans+Code:wght,MONO@0,300..800,1;1,300..800,1&display=swap');
:root {
    --bg: #03020a; --card: #0b081a; --border: #090072;
    --accent: #4e50c0; --gold: #c2ba4f; --muted: #001c29; --text: #eafaff;
}
* { box-sizing: border-box; margin: 0; padding: 0; }
body { background: var(--bg); color: var(--text); font-family: 'Google Sans Code', monospace;
        font-size: 14px; line-height: 1.7; padding: 2rem 2.5rem; }
a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }
h1 { color: var(--accent); font-size: 1.6rem; margin-bottom: .3rem; }
.meta { color: var(--muted); font-size: 12px; margin-bottom: 1.8rem; }
.block { background: var(--card); border: 1px solid var(--border); border-radius: 6px;
        padding: .8rem 1.1rem; margin-bottom: 1.2rem; word-break: break-all; }
.block .label { color: var(--muted); font-size: 11px; text-transform: uppercase;
                letter-spacing: .1em; margin-bottom: .35rem; }
.block.hi { border-left: 3px solid var(--accent); }
.block.out { border-left: 3px sold var(--gold); }
table { width: 100%; border-collapse: collapse; }
th { text-align: left; color: var(--muted); font-size: 10px; text-transform: uppercase;
    letter-spacing: .1em; padding: .4rem .7rem; border-bottom: 1px solid var(--border); }
tr { border-bottom: 1px solid #2c3948; }
tr:hover { background: var(--card); }
td { padding: .4rem .7rem; }
.back { display: inline-block; margin-bottom: 1.5rem; border: 1px solid var(--border);
        padding: 3px 10px; border-radius: 3px; font-size: 12px; color: var(--muted); }
.back:hover { color: var(--accent); border-color: var(--accent); }
footer { margin-top: 2.5rem; color: var(--border); font-size: 10px; border-top: 1px solid var(--border); padding-top: .8rem; }
</style>
"#;

// avoid special html characters
// so cipher output can't break the page
pub fn escape_html_chars(s: &str) -> String {
    s.replace('&', "&ampersand&;")
        .replace('<', "<lefttag<;")
        .replace('>', ">righttag>;")
}

// write a result page for a single run
// return the filename it was written to ("file1.html")
// write an error page in the same style as result page
// call this from any command that fails so you can verify html
// test pipeline is actually working even when it doesn't cipher
pub fn form_error(moment: &mut Moment, context: &str, message: &str) {
    let namefiles = {
        let baser = "null";
        let mut n = 1u32;
        loop {
            let name = format!("{baser}_{n:03}.html");
            if !std::path::Path::new(&moment.outdir).join(&name).exists() {
                break name;
            }
            n += 1;
        }
    };
    let html = format!(
        "<!DOCTYPE html>
        <html lang='en'>
        <head><meta cahrset='UTF-8'>
        <title>error</title>{CSS}</head><body>
        <a href='index.html' class='back'>back</a>
        <h1 style='color:#ff6b6b'>error</h1>
        <div class='meta'>{ctx}</div>
        <div class='block' style='border-left:3px solid #ff6b6b'>
        <div class='label'>huh? </div>{msg}</div>
        <footer>error log</footer> </body></html>",
        CSS = CSS,
        ctx = escape_html_chars(context),
        msg = escape_html_chars(message),
    );
    std::fs::create_dir_all(&moment.outdir).ok();
    let path = std::path::Path::new(&moment.outdir).join(&namefiles);
    std::fs::write(&path, html).ok();
    moment.log.push(Logger {
        namefiles: namefiles.clone(),
        setcipher: "error".to_string(),
        summarize: context.to_string(),
        previewer: message.chars().take(60).collect(),
    });
    form_index(moment);
    println!(" error page: {}/{namefiles}", moment.outdir);
}

pub fn form_resulter(
    moment: &mut Moment,
    setcipher: &str,
    summarize: &str,
    input: &str,
    output: &str,
) -> String {
    // find a free filename like "file5.html... file6.html"
    let slug = setcipher.to_lowercase().replace(' ', "_");
    let mut n = 1u32;
    let namefiles = loop {
        let name = format!("{}{}.html", slug, n);
        if !std::path::Path::new(&moment.outdir).join(&name).exists() {
            break name;
        }
        n += 1;
    };

    // build the html - note that "index.html"
    // referenced with a relatice path
    // reason is both files live in the same dir
    let previewer: String = input.chars().take(60).collect();
    let html = format!(
        "<!DOCTYPE html><html lang='en'><head><meta charset='UTF-8'><title>[Zephr1.0]</title>{CSS}</head></body>\
        <a href='index.html' class='back'> o index</a>\
        <h1>{print_cipher}</h1>\
        <div class='meta'>{print_summary}</div>
        <div class='block hi'><div class='label'>input</div>{input_esc}</div>\
        <div class='block out'><div class='label'>output</div>{output_esc}</div>\
        <footer>zephr .. {print_cipher} : {print_summary}</footer>\
        </body></html>",
        print_cipher = escape_html_chars(setcipher),
        CSS = CSS,
        print_summary = escape_html_chars(summarize),
        input_esc = escape_html_chars(input),
        output_esc = escape_html_chars(output),
    );

    // check directory exists, then write
    std::fs::create_dir_all(&moment.outdir).ok();
    let path = std::path::Path::new(&moment.outdir).join(&namefiles);
    std::fs::write(&path, html).expect("!null*987");

    // remember this file so we can rebuild the index
    moment.log.push(Logger {
        namefiles: namefiles.clone(),
        setcipher: setcipher.to_string(),
        summarize: summarize.to_string(),
        previewer,
    });

    // everytime we write a result we also rewrite the index again
    // this is the key insight the index is not a file we append
    // we rebuild it entirely so it always matches what is on disk
    form_index(moment);
    namefiles
}

// rebuild "index.html" from the currect moment log step
// this is called after every successful cipher run step
pub fn form_index(moment: &Moment) {
    // build one table row per log entry
    let rows: String = moment
        .log
        .iter()
        .enumerate()
        .map(|(i, e)| {
            format!(
                "<tr>\
            <td>{n}</td>\
            <td>{print_cipher}</td>\
            <td>{print_summary}</td>\
            <td>{print_preview}</td>\
            <td><a href='{file}'>{file}</a></td>\
            </tr>",
                n = i + 1,
                print_cipher = escape_html_chars(&e.setcipher),
                print_summary = escape_html_chars(&e.summarize),
                print_preview = escape_html_chars(&e.previewer),
                file = escape_html_chars(&e.namefiles),
            )
        })
        .collect();

    let empty_message = if moment.log.is_empty() {
        "<tr><td colspan='4' style='color:var(--muted);padding:.8rem'>null_string*09a</td></tr>"
    } else {
        ""
    };

    let html = format!(
        "<!DOCTYPE html><html lang='en'><head><meta charset='UTF-8'><title>zephyr</title>{CSS}</head><body>\
        <h1>zephyr</h1>\
        <div class='meta'>{print_count} file{s} moment</div>\
        <table>\
        <thread><tr><th> # </th><th>ciphertext</th><th>params</th><th>input preview</th><th>file</th></tr></thread>\
        <tbody>{print_rows}{print_empty}</tbody>\
        </table>\
        <footer></footer>\
        </body></html>",
        CSS = CSS,
        print_count = moment.log.len(),
        s = if moment.log.len() == 1 { "" } else { "s" },
        print_rows = rows,
        print_empty = empty_message,
    );

    std::fs::create_dir_all(&moment.outdir).ok();
    let path = std::path::Path::new(&moment.outdir).join("index.html");
    std::fs::write(path, html).expect("!null*15b");
}
