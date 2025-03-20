use pacosso::{ParseError, ParseResult, Stream};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::io::Read;

#[derive(Debug, PartialEq)]
pub struct BibEntry {
    pub pubtype: PubType,
    pub key: String,
    pub author: String,
    pub title: String,
    pub date: String,
}

#[allow(dead_code)]
impl BibEntry {
    pub fn empty() -> BibEntry {
        Self {
            pubtype: PubType::Misc,
            key: "".to_string(),
            author: "".to_string(),
            title: "".to_string(),
            date: "".to_string(),
        }
    }
}

impl Display for BibEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}

#[derive(Debug, PartialEq)]
pub enum PubType {
    Book,
    Article,
    Incol,
    Inproc,
    Misc,
}

impl Display for PubType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub fn parse<R: Read>(s: &mut Stream<R>) -> ParseResult<Vec<BibEntry>> {
    s.many_one(bibentry)
}

#[allow(dead_code)]
fn fail<R: Read>(s: &mut Stream<R>, msg: String) -> ParseResult<BibEntry> {
    s.fail(&msg, BibEntry::empty())
}

fn bibentry<R: Read>(s: &mut Stream<R>) -> ParseResult<BibEntry> {
    s.skip_whitespace()?;
    s.byte(b'@')?;
    let pubtype = pubtype(s)?;
    s.skip_whitespace()?;
    body(s, pubtype)
}

fn pubtype<R: Read>(s: &mut Stream<R>) -> ParseResult<PubType> {
    let book = |p: &mut Stream<R>| -> ParseResult<PubType> {
        p.string_ic("book")?;
        Ok(PubType::Book)
    };
    let article = |p: &mut Stream<R>| -> ParseResult<PubType> {
        p.string_ic("article")?;
        Ok(PubType::Article)
    };
    let inproc = |p: &mut Stream<R>| -> ParseResult<PubType> {
        p.string_ic("inproceedings")?;
        Ok(PubType::Inproc)
    };
    let incol = |p: &mut Stream<R>| -> ParseResult<PubType> {
        p.string_ic("incollection")?;
        Ok(PubType::Incol)
    };
    let misc = |p: &mut Stream<R>| -> ParseResult<PubType> {
        p.string_ic("misc")?;
        Ok(PubType::Misc)
    };
    let choices = [book, article, inproc, incol, misc];
    s.choice(&choices[..])
}

fn body<R: Read>(s: &mut Stream<R>, pt: PubType) -> ParseResult<BibEntry> {
    s.skip_whitespace()?;
    s.byte(b'{')?;
    s.skip_whitespace()?;
    let k = citekey(s)?;
    s.byte(b',')?;
    let hs = headers(s)?;
    s.byte(b'}')?;

    Ok(BibEntry {
        pubtype: pt,
        key: k,
        author: if hs.contains_key("author") {
            hs["author"].to_string()
        } else {
            "".to_string()
        },
        title: if hs.contains_key("title") {
            hs["title"].to_string()
        } else {
            "".to_string()
        },
        date: if hs.contains_key("date") {
            hs["date"].to_string()
        } else {
            "".to_string()
        },
    })
}

// The citekey can be any combination of alphanumeric characters including the characters "-", "_", and ":".
fn citekey<R: Read>(s: &mut Stream<R>) -> ParseResult<String> {
    s.skip_whitespace()?;
    let k = alphanum(s, true)?;
    s.skip_whitespace()?;
    Ok(k)
}

fn headers<R: Read>(s: &mut Stream<R>) -> ParseResult<HashMap<String, String>> {
    let mut m = HashMap::new();
    loop {
        let (k, v) = header(s)?;
        let _ = match m.insert(k.clone(), v) {
            Some(_) => {
                return Err(ParseError::Failed(
                    format!("duplicated key '{}' in BibEntry", k.clone()),
                    s.position(),
                ))
            }
            _ => true,
        };
        s.skip_whitespace()?;
        let ch = s.peek_byte()?;
        if ch != b',' {
            break;
        }
        s.byte(b',')?;
    }
    Ok(m)
}

fn header<R: Read>(s: &mut Stream<R>) -> ParseResult<(String, String)> {
    s.skip_whitespace()?;
    let k = alphanum(s, false)?;
    s.skip_whitespace()?;
    s.byte(b'=')?;
    let v = value(s)?;
    Ok((k, v))
}

// the values of field can either be enclosed in { } or " "
// plain numbers do not need to be enclosed
fn value<R: Read>(s: &mut Stream<R>) -> ParseResult<String> {
    s.skip_whitespace()?;
    let b = s.peek_byte()?;
    let closer = if b == b'"' {
        b'"'
    } else if b == b'{' {
        b'}'
    } else if b.is_ascii_digit() {
        b'0'
    } else {
        b'?'
    };
    if closer == b'?' {
        return s.fail(
            &format!("unexpected token {}, '\"' or '{{' expected", b),
            "".to_string(),
        );
    }
    if closer != b'0' {
        s.byte(b)?;
    }
    let v = chars_until_closer(s, closer as char)?;
    if closer != b'0' {
        s.byte(closer)?;
    }
    s.skip_whitespace()?;
    Ok(v)
}

fn alphanum<R: Read>(s: &mut Stream<R>, ext: bool) -> ParseResult<String> {
    let mut v: Vec<char> = Vec::new();
    loop {
        let ch = s.peek_character()?;
        if !ch.is_alphanumeric() {
            if !ext || (ch != '-' && ch != '_' && ch != ':') {
                break;
            }
        }
        s.character(ch)?;
        v.push(ch);
    }
    Ok(v.into_iter().collect())
}

fn chars_until_closer<R: Read>(s: &mut Stream<R>, closer: char) -> ParseResult<String> {
    let mut v: Vec<char> = Vec::new();
    loop {
        let ch = s.peek_character()?;
        if closer == '0' {
            if !ch.is_ascii_digit() {
                break;
            }
        } else if ch == closer {
            break;
        }
        s.character(ch)?;
        if ch != '{' && ch != '}' {
            v.push(ch);
        }
    }
    Ok(v.into_iter().collect())
}

pub fn collect_cites<R: Read>(s: &mut Stream<R>) -> ParseResult<Vec<String>> {
    let mut v = Vec::new();

    loop {
        if eof(s) {
            break;
        }
        let b = s.any_byte()?;
        if b != b'\\' {
            continue;
        }

        if ignore(s) {
            ignore_text(s)?;
            continue;
        }

        if !cite(s) {
            continue;
        }
        s.skip_whitespace()?;

        move_to_citekey(s)?;

        // consider list of citekeys, e.g.
        // \cite{a, b, c}
        s.skip_whitespace()?;
        let mut ks = citekeylist(s)?;
        s.skip_whitespace()?;
        s.byte(b'}')?;

        v.append(&mut ks);
    }

    Ok(v)
}

fn eof<R: Read>(s: &mut Stream<R>) -> bool {
    match s.eof() {
        Ok(()) => true,
        Err(_) => false,
    }
}

fn cite<R: Read>(s: &mut Stream<R>) -> bool {
    match s.string("cite") {
        Ok(()) => true,
        Err(_) => false,
    }
}

fn ignore<R: Read>(s: &mut Stream<R>) -> bool {
    match s.string("ignore") {
        Ok(()) => true,
        Err(_) => false,
    }
}

fn ignore_text<R: Read>(s: &mut Stream<R>) -> ParseResult<()> {
    s.skip_whitespace()?;
    let b = s.any_byte()?;
    if b != b'{' {
        return Ok(());
    }

    let mut count = 1;
    loop {
        let b = s.any_byte()?;
        if b == b'}' {
            count -= 1;
            if count == 0 {
                break;
            }
        } else if b == b'{' {
            count += 1;
        }
    }
    Ok(())
}

fn move_to_citekey<R: Read>(s: &mut Stream<R>) -> ParseResult<()> {
    let mut nest = 0i8;
    loop {
        if eof(s) {
            return s.fail("Cite without key", ());
        }
        let b = s.any_byte()?;
        if b == b'[' {
            nest += 1;
            continue;
        }
        if b == b']' {
            nest -= 1;
            continue;
        }
        if b != b'{' || nest > 0 {
            continue;
        }
        break;
    }
    Ok(())
}

fn citekeylist<R: Read>(s: &mut Stream<R>) -> ParseResult<Vec<String>> {
    let mut v = Vec::new();

    loop {
        s.skip_whitespace()?;
        let k = citekey(s)?;
        s.skip_whitespace()?;

        v.push(k);

        let b = s.peek_byte()?;
        if b == b',' {
            s.byte(b',')?;
            continue;
        }

        break;
    }

    Ok(v)
}

#[cfg(test)]
mod test {
    use super::*;
    use pacosso::{options::Opts, parse_string};

    fn karl() -> BibEntry {
        BibEntry {
            pubtype: PubType::Book,
            key: "capital".to_string(),
            author: "Karl Marx".to_string(),
            title: "Das Kapital".to_string(),
            date: "1867".to_string(),
        }
    }

    fn mao() -> BibEntry {
        BibEntry {
            pubtype: PubType::Book,
            key: "prac".to_string(),
            author: "毛澤東".to_string(),
            title: "On Practice".to_string(),
            date: "1937".to_string(),
        }
    }

    fn wei() -> BibEntry {
        BibEntry {
            pubtype: PubType::Book,
            key: "ideology".to_string(),
            author: "Wei Wei Zhang".to_string(),
            title: "Ideology and Economic Reform".to_string(),
            date: "1996".to_string(),
        }
    }

    #[test]
    fn test_parse_simple_entry_quoted() {
        let s = r#"@book{capital,
            author = "Karl Marx", 
            title = "Das Kapital",
            date = "1867"
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(be) => {
                println!("success: {:?}", be);
                be.len() == 1 && be[0] == karl()
            }
            Err(e) => {
                eprintln!("error: {:?}", e);
                false
            }
        })
    }

    #[test]
    fn test_parse_simple_entry_curly() {
        let s = r#"@book{capital,
            author = "Karl Marx", 
            title = {Das Kapital},
            date = "1867"
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(be) => {
                println!("success: {}", be[0]);
                be.len() == 1 && be[0] == karl()
            }
            Err(e) => {
                eprintln!("error: {:?}", e);
                false
            }
        })
    }

    #[test]
    fn test_parse_simple_entry_mao() {
        let s = r#"@book{prac,
            author = {毛澤東}, 
            title = "On Practice",
            date = "1937"
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(be) => {
                println!("success: {:?}", be);
                be.len() == 1 && be[0] == mao()
            }
            Err(e) => {
                eprintln!("error: {:?}", e);
                false
            }
        })
    }

    #[test]
    fn test_parse_simple_entry_year_no_quote() {
        let s = r#"@book{prac,
            author = {毛澤東}, 
            title = "On Practice",
            date = 1937
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(be) => {
                println!("success: {:?}", be);
                be.len() == 1 && be[0] == mao()
            }
            Err(e) => {
                eprintln!("error: {:?}", e);
                false
            }
        })
    }

    #[test]
    fn test_parse_simple_entry_wsp_key() {
        let s = r#"@book{ prac ,
            author = {毛澤東}, 
            title = "On Practice",
            date = 1937
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(be) => {
                println!("success: {:?}", be);
                be.len() == 1 && be[0] == mao()
            }
            Err(e) => {
                eprintln!("error: {:?}", e);
                false
            }
        })
    }

    #[test]
    fn test_parse_simple_entry_quotes_and_curly() {
        let s = r#"@book{ ideology,
            author = "{Wei Wei} Zhang", 
            title = "Ideology and Economic Reform",
            date = 1996
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(be) => {
                println!("success: {:?} | {:?}", be, wei());
                be.len() == 1 && be[0] == wei()
            }
            Err(e) => {
                eprintln!("error: {:?}", e);
                false
            }
        })
    }

    #[test]
    fn test_fail_author_no_quotes() {
        let s = r#"@book{ ideology,
            author = Wei Wei Zhang, 
            title = "Ideology and Economic Reform",
            date = 1996
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(_) => false,
            Err(_) => true,
        })
    }

    #[test]
    fn test_fail_title_no_quotes() {
        let s = r#"@book{ ideology,
            author = "Wei Wei Zhang", 
            title = Ideology and Economic Reform,
            date = 1996
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(_) => false,
            Err(_) => true,
        })
    }

    #[test]
    fn test_fail_title_no_comma() {
        let s = r#"@book{ ideology,
            author = "Wei Wei Zhang", 
            title = "Ideology and Economic Reform"
            date = 1996
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(_) => false,
            Err(_) => true,
        })
    }

    #[test]
    fn test_fail_author_no_comma() {
        let s = r#"@book{ ideology,
            author = "Wei Wei Zhang"
            title = "Ideology and Economic Reform",
            date = 1996
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(_) => false,
            Err(_) => true,
        })
    }

    #[test]
    fn test_fail_key_no_comma() {
        let s = r#"@book{ ideology
            author = "Wei Wei Zhang",
            title = "Ideology and Economic Reform",
            date = 1996
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(_) => false,
            Err(_) => true,
        })
    }

    #[test]
    fn test_fail_no_key_no_comma() {
        let s = r#"@book{ 
            author = "Wei Wei Zhang",
            title = "Ideology and Economic Reform",
            date = 1996
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(_) => false,
            Err(_) => true,
        })
    }

    //TODO: no key is ok
    #[test]
    fn test_fail_no_key() {
        let s = r#"@book{ ,
            author = "Wei Wei Zhang",
            title = "Ideology and Economic Reform",
            date = 1996
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(be) => {
                println!("success: {:?}", be);
                true
            }
            Err(_) => true,
        })
    }

    #[test]
    fn test_fail_no_value() {
        let s = r#"@book{ 
            author = ,
            title = "Ideology and Economic Reform",
            date = 1996
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(_) => false,
            Err(_) => true,
        })
    }

    #[test]
    fn test_fail_no_key_in_header() {
        let s = r#"@book{ 
            author = "Karl Marx",
            = "Ideology and Economic Reform",
            date = 1996
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(_) => false,
            Err(_) => true,
        })
    }

    #[test]
    fn test_fail_no_pubtype() {
        let s = r#"@{ 
            author = "Karl Marx",
            = "Ideology and Economic Reform",
            date = 1996
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(_) => false,
            Err(_) => true,
        })
    }

    #[test]
    fn test_fail_unknown_pubtype() {
        let s = r#"@illustrierte{ 
            author = "Karl Marx",
            = "Ideology and Economic Reform",
            date = 1996
        }"#;
        assert!(match parse_string(s.to_string(), Opts::default(), parse) {
            Ok(_) => false,
            Err(_) => true,
        })
    }

    #[test]
    fn test_find_simple_cite() {
        let s = "this is some text\\cite{work}. With some more text.";
        assert!(
            match parse_string(s.to_string(), Opts::default(), collect_cites) {
                Ok(cites) => cites.len() == 1 && cites[0] == "work",
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    false
                }
            }
        )
    }

    #[test]
    fn test_find_3_cites() {
        let s = "this is some text\\cite{misc}. With some more text.\\cite{book}. And still\\cite{article} more.";
        assert!(
            match parse_string(s.to_string(), Opts::default(), collect_cites) {
                Ok(cites) =>
                    cites.len() == 3
                        && cites[0] == "misc"
                        && cites[1] == "book"
                        && cites[2] == "article",
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    false
                }
            }
        )
    }

    #[test]
    fn test_find_cite_with_opt() {
        let s = "this is some text\\cite[p. 1]{book}.";
        assert!(
            match parse_string(s.to_string(), Opts::default(), collect_cites) {
                Ok(cites) => {
                    println!("have: {:?}", cites);
                    cites.len() == 1 && cites[0] == "book"
                }
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    false
                }
            }
        )
    }

    #[test]
    fn test_find_3_cites_with_opt() {
        let s = "this is some text\\cite[p. 1]{book}. Still more text to come\\cite[blabla][pp. 100-120]{article}. An more\\cite[]{misc}.";
        assert!(
            match parse_string(s.to_string(), Opts::default(), collect_cites) {
                Ok(cites) => {
                    println!("have: {:?}", cites);
                    cites.len() == 3
                        && cites[0] == "book"
                        && cites[1] == "article"
                        && cites[2] == "misc"
                }
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    false
                }
            }
        )
    }

    #[test]
    fn test_find_nested_cite() {
        let s = "this is some text\\cite[this is {nested}][p. 1]{book}.";
        assert!(
            match parse_string(s.to_string(), Opts::default(), collect_cites) {
                Ok(cites) => {
                    println!("have: {:?}", cites);
                    cites.len() == 1 && cites[0] == "book"
                }
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    false
                }
            }
        )
    }

    #[test]
    fn test_find_3_nested_cites() {
        let s = "this is some text\\cite[p. 1, {a nested comment}]{book}.\
                 Still more text to come\\cite[blabla, \\speech{and sho on}][pp. 100-120]{article}. An more\\cite[]{misc}.";
        assert!(
            match parse_string(s.to_string(), Opts::default(), collect_cites) {
                Ok(cites) => {
                    println!("have: {:?}", cites);
                    cites.len() == 3
                        && cites[0] == "book"
                        && cites[1] == "article"
                        && cites[2] == "misc"
                }
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    false
                }
            }
        )
    }

    #[test]
    fn test_find_multi_cite() {
        let s = "this is some text\\cite[p. 1]{book, article, misc}.";
        assert!(
            match parse_string(s.to_string(), Opts::default(), collect_cites) {
                Ok(cites) => {
                    println!("have: {:?}", cites);
                    cites.len() == 3
                        && cites[0] == "book"
                        && cites[1] == "article"
                        && cites[2] == "misc"
                }
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    false
                }
            }
        )
    }

    #[test]
    fn test_find_multi_cites() {
        let s = "this is some text\\cite[p. 1]{book, article, misc}. And it goes on\\cite{book, inproc}.";
        assert!(
            match parse_string(s.to_string(), Opts::default(), collect_cites) {
                Ok(cites) => {
                    println!("have: {:?}", cites);
                    cites.len() == 5
                        && cites[0] == "book"
                        && cites[1] == "article"
                        && cites[2] == "misc"
                        && cites[3] == "book"
                        && cites[4] == "inproc"
                }
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    false
                }
            }
        )
    }

    #[test]
    fn test_ignore_cite() {
        let s = "this is some text\\ignore{\\cite[p. 1]{book, article, misc}.}";
        assert!(
            match parse_string(s.to_string(), Opts::default(), collect_cites) {
                Ok(cites) => cites.len() == 0,
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    false
                }
            }
        )
    }

    #[test]
    fn test_ignore_cite_read_cite() {
        let s = "this is some text\\ignore{\\cite[p. 1]{book, article, misc}.} and it goes on\\cite[p. 2]{book}";
        assert!(
            match parse_string(s.to_string(), Opts::default(), collect_cites) {
                Ok(cites) => cites.len() == 1 && cites[0] == "book",
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    false
                }
            }
        )
    }

    #[test]
    fn test_fail_infinite_ignore() {
        let s = "this is some text\\ignore{\\cite[p. 1]{book, article, misc}.";
        assert!(
            match parse_string(s.to_string(), Opts::default(), collect_cites) {
                Ok(_) => false,
                Err(_) => true,
            }
        )
    }
}
