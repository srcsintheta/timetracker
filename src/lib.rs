use std::error;
use std::io::{self, Write};
use rusqlite::Connection;

pub mod db;
pub mod tracker;
#[cfg(test)]
mod test;

use rusqlite::Result;

use chrono::{Datelike, Duration, NaiveDate, Local, TimeZone, Timelike};

pub fn print_acts_get_choice(
    db        : &mut Connection,
    activated : bool,
    )
    ->Result<i32, Box<dyn error::Error>>
{
    let activities = db::get_activities(db, activated)?;
    let mut activities_ids : Vec<i32> = Vec::new();
    let mut idstr = String::new();
    let mut idint;
    
    println!("---------------------------------------------------------------");

    println!("ID\tName");

    for activity in activities
    {
        println!("{}\t{}", activity.id, activity.name);
        activities_ids.push(activity.id);
    }

    println!("---------------------------------------------------------------");

    println!("Enter one of the listed activity IDs");
    println!("  'q' to go back to main");
    println!();
    print!("Your input: ");
    io::stdout().flush().unwrap();

    loop
    {
        // take user input
        idstr.clear(); // necessary, read_line() doesn't do this by itself!
        io::stdin().read_line(&mut idstr).expect("Failed to read line");

        // trim and quit if "q"
        idstr = idstr.trim().to_string();
        if idstr == "q" { return Err("Aborted".into()); }

        // parse to int, break if valid
        idint = idstr.as_str().parse().unwrap_or(-1);
        if activities_ids.contains(&idint) { break; }
    }

    Ok(idint)
}

/// running loop when tracker is tracking an activity
pub fn track(db : &mut Connection) -> Result<(), Box<dyn error::Error>>
{
    let idint;

    match print_acts_get_choice(db, true)
    {
        Ok(value) => { idint = value },
        Err(err)  => { eprintln!("{}", err); return Ok(()); }
    }

    let mut totalwork = chrono::Duration::zero();
    let mut totalpaus = chrono::Duration::zero();

    println!("Press Enter to switch between work/break");
    println!("Press q-Enter to end");

    loop
    {
        println!("Started work timer");

        let mut datetime_beg = chrono::Local::now();
        // endloop is a bool indicating whether loop should be stopped
        let mut endloop = crate::tracker::workloop().unwrap();
        let mut datetime_end = chrono::Local::now();
        let mut duration = datetime_end.signed_duration_since(datetime_beg);

        db::enter_into_db(db, &datetime_beg, &datetime_end, idint)?;

        totalwork = totalwork + duration;

        println!("Work time thus far: {:02}:{:02}:{:02}",
                 totalwork.num_hours(),
                 totalwork.num_minutes() % 60,
                 totalwork.num_seconds() % 60);

        if endloop { break };

        println!("Started break timer");

        // endloop is a bool indicating whether loop should be stopped
        datetime_beg = chrono::Local::now();
        endloop = crate::tracker::workloop().unwrap();
        datetime_end = chrono::Local::now();
        duration = datetime_end.signed_duration_since(datetime_beg);

        totalpaus = totalpaus + duration;

        println!("Break time thus far: {:02}:{:02}:{:02}",
                 totalpaus.num_hours(),
                 totalpaus.num_minutes() % 60,
                 totalpaus.num_seconds() % 60);

        if endloop { break };
    }

    println!();
    println!("Total worked:\t{:02}:{:02}:{:02}",
             totalwork.num_hours(),
             totalwork.num_minutes() % 60,
             totalwork.num_seconds() % 60);
    println!("Total paused:\t{:02}:{:02}:{:02}",
             totalpaus.num_hours(),
             totalpaus.num_minutes() % 60,
             totalpaus.num_seconds() % 60);
    println!("Pause percentage: {:.2}%", 
             totalpaus.num_seconds() as f64 / totalwork.num_seconds() as f64);

    Ok(())
}          

/// statistics on data of sql db; only reads from db;
pub fn statsnormal(db : &mut Connection) -> Result<(), Box<dyn error::Error>>
{
    db::stat::printstats(db)?;
    Ok(())
}

pub fn statsyear(db : &mut Connection) -> Result<(), Box<dyn error::Error>>
{
    db::stat::printstats_year(db)?;
    Ok(())
}

/// configure db; (activities and such)
pub fn conf(db : &mut Connection) -> Result<()>
{
    loop
    {
        println!();
        println!("Options: ");
        println!();
        println!("  (a)dd new");
        println!("  (d)eactivate");
        println!("  (r)eactivate");
        println!("  (q)uit (back to main menu)");
        println!();
        print!("Your option: ");
        io::stdout().flush().unwrap();


        let mut opt = String::new();
        io::stdin().read_line(&mut opt).expect("Failed to read line");
        opt = opt.trim().to_string();

        if opt == "a"
        {
            print!("Enter activity name: ");
            io::stdout().flush().unwrap();
            let mut name = String::new();
            io::stdin().read_line(&mut name).expect("Failed to read line");
            name = name.trim().to_string();

            let date = chrono::Local::now().format("%Y-%m-%d").to_string();

            db.execute(
                &format!("INSERT INTO {}
                        (name, added, hourstotal) 
                        VALUES
                        (?1, ?2, ?3)",
                        crate::db::queries::SQL_TABLEN_ACT),
                        rusqlite::params![name, date, 0.]
                      )?;
        }
        else if opt == "d"
        {
            let id;

            match print_acts_get_choice(db, true)
            {
                Ok(value) => { id = value },
                Err(err)  => { eprintln!("{}", err); return Ok(()); }
            }

            let id_lowestinactive : i32 = db.query_row(
                &format!(
                    "SELECT id FROM {} WHERE id < 0 ORDER BY id ASC",
                    db::queries::SQL_TABLEN_ACT), (),
                    |row| row.get(0)).unwrap_or_else(|_| 0);

            // ideally a transaction & rollback should be used here

			db.execute("PRAGMA foreign_keys=OFF;", rusqlite::params![])?;

            db.execute(
                &format!("UPDATE {} SET id = ?1 WHERE id = ?2",
                         crate::db::queries::SQL_TABLEN_ACT),
                         rusqlite::params![id_lowestinactive - 1, id])?;

            db.execute(
                &format!("UPDATE {} SET id = ?1 WHERE id = ?2",
                         crate::db::queries::SQL_TABLEN_HIS),
                         rusqlite::params![id_lowestinactive - 1, id])?;

            // decrement all subsequent IDs in activities and history table

            let mut id = id + 1;

            loop
            {
                // activities table
                let changed = db.execute(
                    &format!("UPDATE {} SET
                             id = ?1 WHERE id = ?2",
                             crate::db::queries::SQL_TABLEN_ACT),
                             rusqlite::params![id-1, id])
                    .map_err(|_| return ()).unwrap();

                if changed == 0 { break };

                // history table
                let _ = db.execute(
                    &format!("UPDATE {} SET
                             id = ?1 WHERE id = ?2",
                             crate::db::queries::SQL_TABLEN_HIS),
                             rusqlite::params![id-1, id])
                    .map_err(|_| return ());

                id += 1;
            }

           db.execute("PRAGMA foreign_keys=ON;", rusqlite::params![])?;

           println!("Activity deactivated");
        }
        else if opt == "r"
        {
            let id;

            match print_acts_get_choice(db, false)
            {
                Ok(value) => { id = value },
                Err(err)  => { eprintln!("{}", err); return Ok(()); }
            }
            
            let id_highestactive : i32 = db.query_row(
                &format!(
                    "SELECT id FROM {} WHERE id > 0 
                    ORDER BY id DESC LIMIT 1",
                    db::queries::SQL_TABLEN_ACT), (),
                    |row| row.get(0)).unwrap_or_else(|_| 0);

			db.execute("PRAGMA foreign_keys=OFF;", rusqlite::params![])?;

            // update activities
            
            db.execute(
                &format!("UPDATE {} SET id=?1 WHERE id = ?2",
                        crate::db::queries::SQL_TABLEN_ACT),
                        rusqlite::params![id_highestactive+1, id])?;

            // update history

            db.execute(
                &format!("UPDATE {} SET id=?1 WHERE id = ?2",
                        crate::db::queries::SQL_TABLEN_HIS),
                        rusqlite::params![id_highestactive+1, id])?;

            // reshuffle the inactive IDs
            
            let mut id = id - 1;

            loop
            {
                // activities table
                let changed = db.execute(
                    &format!("UPDATE {} SET
                             id = ?1 WHERE id = ?2",
                             crate::db::queries::SQL_TABLEN_ACT),
                             rusqlite::params![id+1, id])
                    .map_err(|_| return ()).unwrap();

                if changed == 0 { break };

                // history table
                let _ = db.execute(
                    &format!("UPDATE {} SET
                             id = ?1 WHERE id = ?2",
                             crate::db::queries::SQL_TABLEN_HIS),
                             rusqlite::params![id+1, id])
                    .map_err(|_| return ());

                id -= 1;
            }

           db.execute("PRAGMA foreign_keys=ON;", rusqlite::params![])?;

           println!("Activity reactivated");
        }
        else if opt == "q"
        {
            break;
        }
    }

    Ok(())
}

/// manual db time entry
pub fn manual(db : &mut Connection)
    -> Result<(), Box<dyn error::Error>>
{
    println!("For which activity do you want to add a time?");

    let idint;

    match print_acts_get_choice(db, true)
    {
        Ok(value) => { idint = value },
        Err(err)  => { eprintln!("{}", err); return Ok(()); }
    }

    println!("For which day do you want to enter a time: ");
    println!("  1) Today");
    println!("  2) Yesterday");
    println!("  3) Specify date manually");
    println!("  ('q' to go back to main)");
    println!();
    print!("Your input: ");
    io::stdout().flush().unwrap();

    let mut opt = "".to_string();

    loop
    {
        io::stdin().read_line(&mut opt).expect("Failed to read line");
        opt = opt.trim().to_string();
        if ["1", "2", "3", "q"].contains(&opt.as_str()) { break; }
        opt.clear();
    }

    if opt == "q" { return Ok(()) }

    let mut dtbeg = chrono::Local::now();
    // reset time to midnight
    // this way we make sure when we add the user entered work time
    // we don't overshoot into next day
    dtbeg = dtbeg.with_hour(0).unwrap();
    dtbeg = dtbeg.with_minute(0).unwrap();

    assert_eq!(dtbeg.hour(), 0);
    assert_eq!(dtbeg.minute(), 0);

    let today = chrono::Local::now();
    let todayyear = today.year();
    let todaymonth = today.month();
    let todayday  = today.day();

    let mut year_int;
    let mut month_int;
    let mut day_int;

    if opt == "1"
    {
        /* nothing to do, dtbeg can be used as is */
    }
    else if opt == "2"
    {
        dtbeg -= chrono::Duration::days(1);
    }
    else if opt == "3"
    {
        println!("Enter year, month, day manually: ");
        print!("  Year (2000 - {}): ", dtbeg.year());
        io::stdout().flush().unwrap();

        let mut year = "".to_string();
        year_int = 0;

        while year_int < 2000 || year_int > todayyear
        {
            year.clear();
            io::stdin().read_line(&mut year).expect("Failed to read line");
            year_int = year.trim().parse().unwrap_or(-1);
        }

        print!("  Month (01 - 12): ");
        io::stdout().flush().unwrap();

        let mut month = "".to_string();
        month_int = 0;

        while month_int <= 0 || month_int > 12 ||
            (todayyear == year_int && month_int > todaymonth)
        {
            month.clear();
            io::stdin().read_line(&mut month).expect("Failed to read line");
            month_int = month.trim().parse().unwrap_or(0);
        }

        let maxdays = db::helpers::days_in_month(year_int, month_int);
        print!("  Day (01 - {}): ", maxdays);
        io::stdout().flush().unwrap();

        let mut day = "".to_string();
        day_int = 0;

        while day_int <= 0 || day_int > maxdays ||
            (year_int == todayyear && month_int == todaymonth &&
             day_int > todayday)
             
        {
            day.clear();
            io::stdin().read_line(&mut day).expect("Failed to read line");
            day_int = day.trim().parse().unwrap_or(0);
        }

        dtbeg = chrono::Local
            .with_ymd_and_hms(year_int, month_int, day_int, 0, 0, 0)
            .unwrap();

    }
    else
    {
        panic!("Error w/ entered option, something went wrong");
    }

    println!("Enter duration (hours and minutes): ");

    print!("  Hours   (0-23): ");
    io::stdout().flush().unwrap();
    let mut hours = "".to_string();
    let mut hours_int = -1;

    while hours_int < 0 || hours_int >= 23
    {
        hours.clear();
        io::stdin().read_line(&mut hours).expect("Failed to read line");
        hours_int = hours.trim().parse().unwrap_or(-1);
    }

    print!("  Minutes (0-59): ");
    io::stdout().flush().unwrap();
    let mut minutes = "".to_string();
    let mut minutes_int = -1;

    while minutes_int < 0 || minutes_int >= 60
    {
        minutes.clear();
        io::stdin().read_line(&mut minutes).expect("Failed to read line");
        minutes_int = minutes.trim().parse().unwrap_or(-1);
    }

    println!("---------------------------------------------------------------");
    println!("Confirm your entry!");
    println!("  Duration: {} hours and {} minutes", hours_int, minutes_int);
    println!("  Day&Date: {}, {}", dtbeg.weekday(), dtbeg.format("%Y-%m-%d"));
    println!("  Activity: {}", db::get_activityname_for_id(db, idint).unwrap());
    println!("---------------------------------------------------------------");
    print!("Is above information correct? (y/n): ");
    io::stdout().flush().unwrap();
    let mut choice : String = Default::default();

    loop
    {
        choice.clear();
        io::stdin().read_line(&mut choice).expect("Failed to read line");
        if ["y", "n"].contains(&choice.trim()) { break; }
    }

    println!("---------------------------------------------------------------");

    // dtbeg should be correct date set to midnight
    // hours and minutes should be max 23 59
    // all that's left is to construct dtend and pass unto db entry function

    assert!(hours_int >= 0 && hours_int <= 23);
    assert!(minutes_int >= 0 && minutes_int <= 59);

    let dtend = dtbeg + 
        Duration::hours(hours_int) + Duration::minutes(minutes_int);

    db::enter_into_db(db, &dtbeg, &dtend, idint)?;

    println!("Your entry has successfully been added");
    println!();

    Ok(())
}

pub fn delete(db : &mut Connection) -> Result<(), Box<dyn error::Error>>
{
    println!();
    println!("Entry deletion supported for today and up to 7 days prior");
    println!("---------------------------------------------------------------");

    let historyvec = db::retrieve_8day_history(db).unwrap();

    let mut validindexes : Vec<i32> = Vec::new();

    for (index, entry) in historyvec.iter().enumerate()
    {
        let naivedate = NaiveDate::parse_from_str(
            entry.date.as_str(), "%Y-%m-%d")?;
        let weekday = naivedate.weekday();

        println!("#{}\tDate: {} {}, hours: {:5.2}, Activity: {}",
                 index,
                 weekday,
                 entry.date, entry.hours,
                 db::get_activityname_for_id(db, entry.id).unwrap(),
                 );

        validindexes.push(index as i32);
    }
    println!("---------------------------------------------------------------");

    println!();
    println!("Specify a valid entry number");
    println!("  'q' to go back to main");
    print!("Your input: #");
    io::stdout().flush().unwrap();

    let mut indexstr = "".to_string();
    let mut index;

    loop
    {
        indexstr.clear();
        io::stdin().read_line(&mut indexstr).expect("Failed to read line");
        if indexstr.trim() == "q" { return Ok(()); }

        index = indexstr.trim().parse().unwrap_or(-1);

        if validindexes.contains(&index) { break };
    }

    assert!(index >= 0);

    // create date for removal function

    let index   = index as usize;
    let datestr = historyvec[index].date.clone();
    let id 	    = historyvec[index].id;

    // since the datestr is retrieved from the history table
    // I assume its correctness here (lot of .unwrap())

    let parts : Vec<&str> = datestr.split('-').collect();
    let year  : i32 = parts.get(0).unwrap().parse().unwrap();
    let month : u32 = parts.get(1).unwrap().parse().unwrap();
    let day   : u32 = parts.get(2).unwrap().parse().unwrap();

    let datetime = Local.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();

    // user confirmation
    println!("---------------------------------------------------------------");
    println!("Sure you want to remove the following entry: ");
    let naivedate = NaiveDate::parse_from_str(datestr.as_str(), "%Y-%m-%d")?;
    let weekday   = naivedate.weekday();
    let actname   = db::get_activityname_for_id(db, historyvec[index].id)
        .unwrap();

    println!("#{}\tDate: {} {}, hours: {:.6}, Activity: {}",
             index, weekday, datestr,
             historyvec[index].hours,
             actname,
             );
    println!("---------------------------------------------------------------");
    print!("Should the above entry be removed? (y/n): ");
    io::stdout().flush().unwrap();
    let mut choice = "".to_string();

    loop
    {
        choice.clear();
        io::stdin().read_line(&mut choice).expect("Failed to read line");
        choice = choice.trim().to_string();
        if ["y", "n"].contains(&choice.as_str()) { break; }
    }
    println!("---------------------------------------------------------------");

    if choice == "n"
    {
        println!("Have not removed entry, back to main menu");
    	println!("-------------------------------------------------------------
                 --");
        return Ok(());
    }

    db::remove_from_db(db, &datetime, id)?;

    println!("Entry removed");
    println!("---------------------------------------------------------------");


    Ok(())
}

/// end of program routine
pub fn quit()
{
    std::process::exit(0);
}
