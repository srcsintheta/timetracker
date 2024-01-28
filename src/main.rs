use std::error;
use std::fs;
use std::io;
use std::io::Write;
use std::path;

use timetracker::db;
use directories::ProjectDirs;
use rusqlite::Connection;

const VERSION: &str = "0.1.0"; // keep in synch w/ ver from Cargo.toml
const DB_NAME: &str = "productivity.db";

fn main() -> Result<(), Box<dyn error::Error>>
{
    // retrieve OS specific configuration folder (eg `~/.config` for unix)
    // using directories module for OS specific configuration path
    let projdir = ProjectDirs::from("dev", "sintheta", "timetracker");

    let dcpath: path::PathBuf; // directory path
    let dbpath: path::PathBuf; // database file path

    let dcpath_exists: bool;
    let dbpath_exists: bool;

    println!();

    if let Some(d) = projdir
    {
        dcpath = d.config_dir().to_path_buf();
        dbpath = dcpath.join(DB_NAME);
    }
    else 
    {
        panic!("Could not retrieve OS specific configuration folder!");
    }

    dcpath_exists = dcpath.exists();
    dbpath_exists = dbpath.exists();

    if !dcpath_exists
    {
        println!("folder  doesn't exist, creating: {:?}", dcpath);
        fs::create_dir_all(&dcpath)?;
    }
    if !dbpath_exists
    {
        println!("db file doesn't exist, creating: {:?}", dbpath);
        // creation below via Connection::open (creates if it doesn't exist)
    }

    println!("Productivity tracker");
    println!("Version : {}", VERSION);
    println!("Database used: {:?}", dbpath);

    let mut db = Connection::open(dbpath)?; // create/open db
    if !dbpath_exists
    {
        println!("initializing db w/ needed tables");
        db::init(&mut db)?;
    }

    /* BEGIN
     * db::check()
     *      a) checks integrity, if tables not detected
     *      b) calls db::init() for initilization
     *      c) which calls crate::conf() so user sets up activities etc
     * whether db is new or not, all we do is
     */
    timetracker::db::check(&db)?;
    /* END */


    loop
    {
        println!();
        println!("-----------------");
        println!("--- Main Menu --- ");
        println!("-----------------");
        println!("Available options");
        println!();
        println!("  1) track");
        println!("  2) manual entry");
        println!("  3) delete entry");
        println!();
        println!("  4) stats");
        println!("  5) stats (yearly)");
        println!();
        println!("  6) configuration of activities");
        println!("  7) exit");
        println!();
        print!("Your option: ");
        io::stdout().flush().unwrap();

        let mut option = String::new();
        io::stdin().read_line(&mut option).expect("Failed to read line");
        option = option.trim().to_string();

        println!();

        match option.as_str() {
            "1" => timetracker::track(&mut db)?,
            "2" => timetracker::manual(&mut db)?,
            "3" => timetracker::delete(&mut db)?,
            "4" => timetracker::statsnormal(&mut db)?,
            "5" => timetracker::statsyear(&mut db)?,
            "6" => timetracker::conf(&mut db)?,
            "7" => timetracker::quit(),
            _ => (),
        }
    }
}
