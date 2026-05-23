// ..REPL
//
// standalone function rather than a closure so it doesn't hold
// a borrow on on session moments for the whole loop body
// a closure capturing '&moment' would conflict with the '&mut'
// session moment that 'form_resutler' needs later on in the same scope
// rust will not allow both at once, even if they do not overlap at runtime
//
// priority is: /text flag > /file flag > file.text
// added: file_read() is a safe file reader with size guard
// checks /input/ dir first then treats the path as-is
// returns Err with a clear message if the file is too large or null
//
// a size limit matters: fs::read_to_string() loads
// the WHOLE file into RAM as one String... 500MB file
// would freeze the process, this guard checks metadata
// (just a syscall, no read on content) before loading it

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

pub fn repl() {
    let mut moment = Moment::new();

    // changed this create both dirs up front so they'll
    // always be there no matter what...
    // '/input/' is self-explainatory
    // '/output/html/' keeps html away from project root?
    fs::create_dir_all(&moment.outdir).ok();
    fs::create_dir_all("input").ok();

    println!(" zephyr .. type 'help' to init...");
    println!(" writes to {}/index.html\n", moment.outdir);

    let sin = io::stdin();
    prompt();

    for raw in sin.lock().lines() {
        // bug fix here was Ok(1) is a [integer literal]
        // must be Ok(l) in this case [variable binding]
        // .trim() also returns a &str so we call .to_string()
        // this becomes a fast single step, no shadows...
        let liner: String = match raw {
            Ok(l) => l.trim().to_string(),
            Err(_) => break,
        };
        if liner.is_empty() || liner.starts_with('#') {
            prompt();
            continue;
        }
        let f = Flags::parser(&liner);

        // ..Commands
        //

        if f.command == "help" {
            print_help();
        }

        else if f.command == "qq!" {
            println!("");
            break;
        }

        else if f.command == "debug" {
            // prints exactly what the parser sees
            // use this when something is silently failing
            // to understand why a flag isn't being read
            println!(" command: {:?}", f.command);
            println!("   flags: {:?}", f.map);
            println!("position: {:?}", f.position);
            println!("    text: {:?}", moment.text.as_deref().map(|t| &t[..60.min(t.len())]));
            let craft_debug = format!(
                "command={:?} flags={:?} position={:?}",
                f.command, f.map, f.position
            );
            let namer = form_resulter(
                &mut moment,
                "debug",
                "debug files",
                "internal",
                &craft_debug,
            );
            println!(" debugger page {}/{namer}", moment.outdir);
        }

        else if f.command == "read" {
            // added: "read file.txt" loads file into session text
            // looks in /input/ first, then falls back to path as-is
            // creates /input/ with a hint messahe if it doesn't exist
            //  ^ fallback since the dir is made every time the prog runs
            // usage:   read notes.txt    if notes.txt is in /input/
            //          read /full_path/file.txt    (absolute path)
            let path = if f.position.is_empty() {
                f.read_get("file", "").to_string()  // file.txt
            } else {
                f.check_position()                  // "my_notes.txt, etc..."
            };
            if path.is_empty() {
                println!(" read file.txt");
                println!(" place in /input folder");
            } else {
                // create /input/ dir if missing show hint
                if !Path::new("input").exists() {
                    fs::create_dir_all("input").ok();
                    println!("");
                }
                match file_read(&path) {
                    Ok(t) => {
                        println!(
                            "loading {} characters {}",
                            t.len(),
                            t.chars().take(50).collect::<String>()
                        );
                        moment.text = Some(t);
                    }
                    Err(e) => {
                        println!("null*{e}");
                        form_error(&mut moment, &format!("read {path}"), &e);
                    }
                }
            }
        }

        else if f.command == "set" {
            // three ways to set text ...
            //      set this is a text message      <-  position of words (no quotes needed)
            //      set /t or /text thismessage     <-  /t flag          (whole string only)
            //      set /f or /file /a/b/c/n.txt    <-  reads file from disk
            match get_txt(&f, &moment) {
                Ok(t) => {
                    println!("set ({} chars)", t.len());
                    moment.text = Some(t);
                }
                Err(e) => println!(" err: {e}"),
            }
        }

        else if f.command == "show" {
            let previewer = moment.text.as_deref()
                .map(|t| {
                    let chars: String = t.chars().take(60).collect();
                    if t.chars().count() > 60 { format!("{chars}...") }
                    else { chars }
                }).unwrap_or_else(|| "(null)".to_string());
            println!("\t text:   {previewer}");
            println!("\t outdir: {}", moment.outdir);
            println!("\t output: {}", moment.log.len());
        }

        else if f.command == "out" {
            let dir = f.read_get("dir", "");
            if dir.is_empty() {
                println!(" out --dir path");
            } else {
                moment.outdir = dir.to_string();
                println!(" out -> {dir}");
            }
        }

        else if f.command == "csr" {
            match get_txt(&f, &moment) {
                Err(e) => println!(" err: {e}"),
                Ok(t) => {
                    let decode = f.flag_off("decode");
                    let shift = f.read_num("shift", 0);
                    let result = use_caesar(&t, if decode { -shift } else { shift });
                    let summarize = format!("shift={shift}{}", if decode { ", decode" } else { "" });
                    let file = form_resulter(&mut moment, "Caesar", &summarize, &t, &result);
                    println!("\t -> {}/{file}", moment.outdir);
                    println!("\t -> {}/index.html saved!", moment.outdir);
                }
            }
        }

        else if f.command == "vig" {
            match get_txt(&f, &moment) {
                Err(e) => println!(" err {e}"),
                Ok(t) => {
                    let key = f.read_get("key", "");
                    let decode = f.flag_off("decode");
                    if key.is_empty() {
                        println!(" vig --key default");
                    } else {
                        let result = use_vigenere(&t, key, decode);
                        let summarize =
                            format!("key={key}{}", if decode { ", decode" } else { "" });
                        let file = form_resulter(&mut moment, "Vigenere", &summarize, &t, &result);
                        println!("\t -> {}/{file}", moment.outdir);
                    }
                }
            }
        }

        else if f.command == "auto" {
            match get_txt(&f, &moment) {
                Err(e) => println!(" err {e}"),
                Ok(t) => {
                    let primer = f.read_get("primer", "");
                    let decode = f.flag_off("decode");
                    if primer.is_empty() {
                        println!("");
                    } else {
                        let result = use_autokey(&t, primer, decode);
                        let summarize =
                            format!("primer={primer}{}", if decode { ", decode" } else { "" });
                        let file = form_resulter(&mut moment, "auto", &summarize, &t, &result);
                        println!(" -> {}/{file}", moment.outdir);
                    }
                }
            }
        }

        else if f.command == "bfort" {
            match get_txt(&f, &moment) {
                Err(e) => println!(" err {e}"),
                Ok(t) => {
                    let key = f.read_get("key", "");
                    if key.is_empty() {
                        println!(" b --key word");
                    } else {
                        let result = use_beaufort(&t, key);
                        let summarize = format!("key={key}");
                        let file = form_resulter(&mut moment, "b", &summarize, &t, &result);
                        println!(" -> {}/{file}", moment.outdir);
                    }
                }
            }
        }

        else if f.command == "sub" {
            match get_txt(&f, &moment) {
                Err(e) => println!(" err {e}"),
                Ok(t) => {
                    let key = f.read_get("key", "");
                    let decode = f.flag_off("decode");
                    let result = use_substitute(&t, key, decode);
                    // use_sub..() returns an error on bad key
                    // not ideal but keeps the return type simple
                    let summarize = format!(
                        "key={}...{}",
                        &key[..4.min(key.len())],
                        if decode { ", decode" } else { "" }
                    );
                    let file = form_resulter(&mut moment, "sub", &summarize, &t, &result);
                    println!(" -> {}/{file}", moment.outdir);
                }
            }
        }

        else if f.command == "word" {
            match get_txt(&f, &moment) {
                Err(e) => println!(" err {e}"),
                Ok(t) => {
                    let kw = f.read_get("key", "");
                    let decode = f.flag_off("decode");
                    if kw.is_empty() {
                        println!(" word --key word");
                    } else {
                        let alpha = set_kword_alpha(kw);
                        let result = use_substitute(&t, &alpha, decode);
                        let summarize = format!("kw={kw} -> {alpha}");
                        let file = form_resulter(&mut moment, "kw", &summarize, &t, &result);
                        println!(" -> {}/{file}", moment.outdir);
                    }
                }
            }
        }

        else if f.command == "trans" {
            match get_txt(&f, &moment) {
                Err(e) => println!(" err {e}"),
                Ok(t) => {
                    // single colmumn count
                    if !f.read_get("n", "").is_empty() {
                        let n: usize = f.read_get("n", "2").parse().unwrap_or(2);
                        let result = use_transposition(&t, n);
                        let summarize = format!("n={n}");
                        let file = form_resulter(&mut moment, "trans", &summarize, &t, &result);
                        println!(" -> {}/{file}", moment.outdir);
                    } else {
                        // range sweep .. produces one file w/ table and results
                        let (from, to) = f.read_range();
                        let rows: String = (from..=to).map(|n| {
                            let out = use_transposition(&t, n);
                            let same = out == t;
                            format!(
                                "<tr{}><td style='color:var(--accent)'>n={n}</td>\
                                <td style=color:var(--muted)'>{previewer}</td>\
                                <td>{out}{dim}</td></tr>",
                                if same { " style='opacity:.3'" } else { "" },
                                previewer = escape_html_chars(&t.chars().take(30).collect::<String>()),
                                out = escape_html_chars(&out),
                                dim = if same { " <span style='color:var(--muted);font-size:10px'>ident</span>" } else { "" },
                            )
                        }).collect();

                        // build the page inline since range output is different from one result
                        let summarize = format!("ranges={from}..{to}");
                        let slug = format!("t_range={:03}", moment.log.len() + 1);
                        let namefiles = format!("{slug}.html");
                        let previewer: String = t.chars().take(60).collect();

                        let html = format!(
                            "<!DOCTYPE html><html lang='en'><head><meta charset='UTF-8'>\
                            <title>transposer</title>{CSS}</head><body>\
                            <a href='index.html' class='back'><- indexer</a>\
                            <h1>transposer</h1>\
                            <div class='meta'>{print_summary}</div>\
                            <div class='block hi'><div class='label'>inputs</div>{input_esc}</div>\
                            <table><thead><tr><th>n</th><th>input</th><th>output</th></tr></thead>\
                            <tbody>{print_rows}</tbody></table>\
                            <footer>zephr transposer {print_summary}</footer>\
                            </body></html>",
                            CSS = CSS,
                            print_summary = escape_html_chars(&summarize),
                            input_esc = escape_html_chars(&t),
                            print_rows = rows,
                        );

                        fs::create_dir_all(&moment.outdir).ok();
                        let path = Path::new(&moment.outdir).join(&namefiles);
                        fs::write(&path, html).expect("null*50567");

                        moment.log.push(Logger {
                            namefiles: namefiles.clone(),
                            setcipher: "trans".to_string(),
                            summarize: summarize.clone(),
                            previewer,
                        });
                        form_index(&moment);
                        println!(" -> {}/{namefiles}", moment.outdir);
                        println!(" -> {}/index.html written!", moment.outdir);
                    }
                }
            }
        } else {
            println!(" blank '{}' none value use 'help me' ", f.command);
        }

        prompt();
    }
}

fn prompt() {
    print!(" ..> ");
    let _ = io::stdout().flush();
}

fn print_help() {
    println!(
        r#"
        nothing right now...
    "#
    );
}