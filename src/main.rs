
// nodaf
// two simple commands
// 		- new [name] create a note with a specific name // newidd [enw]
//		- get [name] retrieve a specific note			// gafael [enw]

// How big would a ramble be? 32KB? Imma use that for tf/idf

use std::env;
use std::io::{self, Write, stdout};
use std::path::{PathBuf, Path};
use std::fs;
use std::process;

// for search use `basedir` + .index
fn main() -> io::Result<()> {
	// trio creu `basedir` o XDG_DATA_HOME.
	let basedir = match env::var_os("XDG_DATA_HOME") {
		Some(x) => x,
		None    => {
			println!("$XDG_DATA_HOME heb ei osod");
			println!("$XDG_DATA_HOME not set");
			return Ok(());
		}
	};
	let mut basedir = PathBuf::from(basedir);
	basedir.push("nodaf");
	// gwneud siwr bod `basedir` yn bodoli
	if !basedir.exists() {
		println!("dydy ffoldr '{}' ddim yn bodoli, creu'r ffoldr",
				 basedir.display());
		fs::create_dir(&basedir)?;
	}
	let argv: Vec<String> = env::args().collect();
	if argv.len() < 2 {
		println!("rowch orchymyn os gwelwch yn dda.");
		println!("Please provide a command");
		return Ok(());
	}
	'selection: { match argv[1].as_str() {
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
		x => {
			println!("gorchymyn heb gydnabod '{}'", x);
			println!("command '{}' not recognised", x);
		}
	} }
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
	process::Command::new("nano")
		    	 	 .arg(targed)
		    		 .spawn()?
		    		 .wait()?;
	
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
