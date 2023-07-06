// nodaf
// two simple commands
// 		- new [name] create a note with a specific name // newidd [enw]
//		- get [name] retrieve a specific note			// gafael [enw]

// How big would a ramble be? 32KB? Imma use that for tf/idf

use std::collections::HashMap;
use std::io::{self, stdout, Write};
use std::path::{Path, PathBuf};
use std::{env, fs, process, str};

macro_rules! unwrap_or_drop {
    ($x:expr) => {
        if $x.is_err() {
            return false;
        } else {
            $x.unwrap()
        }
    };
}
// for search use `basedir` + .index
fn main() -> io::Result<()> {
    // trio creu `basedir` o XDG_DATA_HOME.
    let basedir = match env::var_os("XDG_DATA_HOME") {
        Some(x) => x,
        None => {
            println!("$XDG_DATA_HOME heb ei osod");
            println!("$XDG_DATA_HOME not set");
            return Ok(());
        }
    };
    let mut basedir = PathBuf::from(basedir);
    basedir.push("nodaf");
    // gwneud siwr bod `basedir` yn bodoli
    if !basedir.exists() {
        println!(
            "dydy ffoldr '{}' ddim yn bodoli, creu'r ffoldr",
            basedir.display()
        );
        fs::create_dir(&basedir)?;
    }
    let argv: Vec<String> = env::args().collect();
    if argv.len() < 2 {
        println!("rowch orchymyn os gwelwch yn dda.");
        println!("Please provide a command");
        return Ok(());
    }
    'selection: {
        match argv[1].as_str() {
            "get" | "gafael" => {
                if argv.len() < 3 {
                    println!("rowch enw nodyn os gwelwch yn dda");
                    println!("please provide note name");
                    break 'selection;
                }
                nodain_get(&argv[2], &basedir)?;
            }
            "new" | "newidd" => {
                if argv.len() < 3 {
                    println!("rowch enw nodyn os gwelwch yn dda");
                    println!("please provide a note name");
                    break 'selection;
                }
                nodain_new(&argv[2], &basedir)?;
            }
            "test" => {
                // fucking indexing shit.
                let mut corpus = Corpus::new(Vec::new());
                let paths = std::fs::read_dir(&basedir)?.filter(|x| {
                    let thingy = unwrap_or_drop!(x.as_ref()).metadata();
                    let m = unwrap_or_drop!(thingy.as_ref());
                    m.file_type().is_file()
                        && unwrap_or_drop!(x.as_ref())
                            .path()
                            .extension()
                            .map(|c| c == "nod")
                            .unwrap_or(false)
                });
                for file in paths.map(|x| x.map(|x| x.path())) {
                    let file = if let Ok(sfile) = file.as_ref() {
                        sfile
                    } else {
                        eprintln!("ERROR: {}", file.as_ref().unwrap_err());
                        return Err(io::Error::new(io::ErrorKind::Other, file.unwrap_err()));
                    };
                    let data = unsafe { String::from_utf8_unchecked(fs::read(file)?) };

                    if let Some(doc) = Document::new(file.to_str().unwrap().to_string(), data) {
                        corpus.1.push(doc);
                    } else {
                        eprintln!("Cannot open note {:?}", file);
                    }
                }
                println!("{corpus:?}");
            }
            x => {
                println!("gorchymyn heb gydnabod '{}'", x);
                println!("command '{}' not recognised", x);
            }
        }
    }
    Ok(())
}

fn nodain_new(enw: &str, sylfaen: &Path) -> io::Result<()> {
    // check if it doesn't exist [if it does then return]
    let mut targed = sylfaen.join(enw);
    targed.set_extension("nod");
    if targed.exists() {
        println!("nodyn o enw '{}' yn bodoli", enw);
        println!("note of name '{}' exists", enw);
        return Ok(());
    }
    // creu'r ffeil ac agor nano
    // wedyn aros iddo gorffen.
    // TODO: gwneud hyn parchu `$EDITOR`
    let mut file = fs::File::create(&targed)?;
    let _ = file.write(b"\nysgrifennwch eich nodyn yma")?;
    file.sync_all()?;
    process::Command::new("nano").arg(targed).spawn()?.wait()?;

    Ok(())
}

fn nodain_get(enw: &str, sylfaen: &Path) -> io::Result<()> {
    let mut targed = sylfaen.join(enw);
    targed.set_extension("nod");
    if !targed.exists() {
        println!("dydy '{}' ddim yn bodoli", targed.display());
        return Ok(());
    }
    // Nawr, ni'n angen arddangos yr nodyn.
    io::copy(&mut fs::File::open(&targed)?, &mut stdout())?;
    Ok(())
}

struct Tocynnudd<'a> {
    sylfaen: &'a str,
    ffynhonell: &'a [u8],
    gwth: usize,
}

impl<'a> Tocynnudd<'a> {
    fn new(sylfaen: &'a str) -> Self {
        Tocynnudd {
            sylfaen,
            ffynhonell: sylfaen.as_bytes(),
            gwth: 0,
        }
    }
}

fn utf8_size(me: u8) -> Option<u32> {
    if me & 0x80u8 == 0 {
        return Some(1);
    }
    if me & 0xc0u8 == 0x80u8 {
        return None;
    }
    Some(me.leading_ones())
}

fn utf8_from_bytes(me: &[u8]) -> char {
    // beth os me.len() == 0?
    if me.len() == 1 {
        return me[0] as char;
    }
    let mut out = (me[0] & (0xffu8 >> me.len())) as u32;
    for i in me.iter().skip(1) {
        out <<= 6;
        out += (i & 0x3f) as u32;
    }
    unsafe { char::from_u32_unchecked(out) }
}

fn is_utf8_start(me: u8) -> bool {
    me & 0x80u8 == 0 || me & 0xc0u8 != 0x80u8
}

fn cam_mlaen(me: &Tocynnudd<'_>) -> Option<(char, u32)> {
    let hyd = utf8_size(*me.ffynhonell.get(me.gwth)?)?;
    assert!(hyd > 0, "indecs {}:{:08b}", me.gwth, me.ffynhonell[me.gwth]);
    let temp = utf8_from_bytes(&me.ffynhonell[me.gwth..me.gwth + hyd as usize]);
    Some((temp, hyd))
}

fn toc_skip(me: &mut Tocynnudd<'_>) {
    while let Some((toc, hyd)) = cam_mlaen(me) {
        if toc.is_alphanumeric() {
            break;
        }
        me.gwth += hyd as usize;
    }
}

impl<'a> Iterator for Tocynnudd<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        toc_skip(self);
        if self.gwth >= self.ffynhonell.len() {
            return None;
        }
        let dech = self.gwth;
        while let Some((toc, hyd)) = cam_mlaen(self) {
            if !toc.is_alphanumeric() {
                break;
            }
            self.gwth += hyd as usize;
        }
        Some(&self.sylfaen[dech..self.gwth])
    }
}

// fully in english because why not
#[derive(Debug)]
pub struct Document {
    path: String,
    title: String,
    index: HashMap<String, u32>,
}
impl Document {
    fn new(path: String, source: String) -> Option<Self> {
        let title_len = source.lines().next()?.len();
        let title = String::from(&source[..title_len]);
        let index = term_freq(&source);
        Some(Document { path, title, index })
    }
}
fn term_freq(source: &str) -> HashMap<String, u32> {
    let mut out = HashMap::new();
    for term in Tocynnudd::new(source).filter(|x| x.len() > 2) {
        if let Some(entry) = out.get_mut(term) {
            *entry += 1;
        } else {
            out.insert(term.to_owned(), 1);
        }
    }
    out
}

#[derive(Debug)]
pub struct Corpus(HashMap<String, u32>, Vec<Document>);

impl Corpus {
    fn new(base: Vec<Document>) -> Self {
        let index = doc_freq(&base);
        Corpus(index, base)
    }
}

fn doc_freq<'a>(docs: &'a [Document]) -> HashMap<String, u32> {
    let mut out = HashMap::new();
    for doc in docs {
        for key in doc.index.keys() {
            if let Some(val) = out.get_mut(key) {
                *val += 1;
            } else {
                out.insert(key.clone(), 1u32);
            }
        }
    }
    out
}
mod magic {
    use super::*;
    use std::ptr;
    fn fuckyou<'a, I: 'a, T>(x: I) -> &'a T {
        unsafe { *ptr::addr_of!(x).cast::<&T>() }
    }
    pub fn score_doc(doc: &Document, terms: &[&str], corpus: &Corpus) -> f64 {
        let mut score = 0f64;
        for term in terms {
            let nt = doc
                .index
                .get(fuckyou::<&str, String>(term))
                .copied()
                .unwrap_or(0u32) as f64;
            let tt = doc.index.values().copied().sum::<u32>() as f64;
            let tf = nt / tt;

            let idf = f64::log10(
                corpus.1.len() as f64
                    / (1 + corpus
                        .0
                        .get(fuckyou::<&str, String>(term))
                        .copied()
                        .unwrap_or(0)) as f64,
            );
            score += tf / idf;
        }
        score
    }
}
use magic::score_doc;
