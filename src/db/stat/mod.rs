//! submodule dealing with statistics
//! their creation based on db data, displaying etc

pub mod helpers;

use std::error;
use std::io;
use std::io::Write;

use chrono::Datelike;
use rusqlite::Connection;
use helpers::*;

pub fn printstats(db : &Connection)
    -> Result<(), Box<dyn error::Error>>
{
    let now = chrono::Local::now();

    // retrieve highest active id (we'll iterate up to max ids)

    let id_highestactive : i32 = db.query_row(
        &format!(
            "SELECT id FROM {} WHERE id > 0 
            ORDER BY id DESC LIMIT 1",
            crate::db::queries::SQL_TABLEN_ACT), (),
            |row| row.get(0)).unwrap_or(0);

    if id_highestactive == 0
    {
        println!("No activities are configured");
        return Ok(());
    }

    if db.query_row(&format!("SELECT * FROM {} LIMIT 1",
                    crate::db::queries::SQL_TABLEN_HIS), (),
                    |row| row.get(0)).unwrap_or(0) == 0
        {
            println!("No entries in history table");
            println!("Back to main");
            return Ok(());
        }


    // vectors to store our results in
    // hour totals for every activity in db
    let mut activitynames   : Vec<String> = Vec::new();
    let mut week_tot        : Vec<f64> = Vec::new();
    let mut todaytot        : Vec<f64> = Vec::new();
    let mut last5ddtot		: Vec<f64> = Vec::new();
    let mut last1wktot 		: Vec<(f64, i64)> = Vec::new();
    let mut last6wktot 		: Vec<(f64, i64)> = Vec::new();
    let mut monthtot 		: Vec<f64> = Vec::new();

    // iterate over activities, use helper functions to retrieve values

    for i in 0..id_highestactive
    {
        let id = i + 1;
        week_tot.push(retrieve_total_this_week(db, now, id).unwrap());
        todaytot.push(retrieve_total_today(db, now, id).unwrap());
        last5ddtot.push(retrieve_total_last_x_days(db, 5, now, id).unwrap());
        last1wktot.push(retrieve_total_last_x_weeks(db, 1, now, id).unwrap());
        last6wktot.push(retrieve_total_last_x_weeks(db, 6, now, id).unwrap());
        monthtot.push(retrieve_total_this_month(db, now, id).unwrap());

        activitynames.push(
            db.query_row(
                &format!("SELECT name FROM {} WHERE id = ?",
                         crate::db::queries::SQL_TABLEN_ACT),
                         rusqlite::params![id], 
                         |row| row.get(0)).unwrap()
            );
    }

    dbg!(&last5ddtot);

    // compute the total and avg values (for all activites in our vectors)

    // week
    let weektotalsum : f64 = week_tot.iter().sum();
    let daysrelevant : i32;

    // in the very very first week discount days before first entry
    let firstentry = firstentry_datetime(db)?;
    if firstentry.year() == now.year() &&
        firstentry.iso_week().week() == now.iso_week().week()
    {
        daysrelevant =
            // week day number excluding today
            (now.weekday().number_from_monday() as i32  - 1) -
            // minus leading days to first entry
            // (not subtracting actual day of first entry, therefore +1 here)
            (firstentry.weekday().number_from_monday() as i32 - 1);
    }
    else
    {
        daysrelevant = now.weekday().number_from_monday() as i32 - 1;
    }
    let weektotalavg = weektotalsum / daysrelevant as f64;

    // today

    let todaytotallsum : f64 = todaytot.iter().sum();

    // last5days
    let     last5ddtotsum    : f64 = last5ddtot.iter().sum();
    let mut last5ddtotdivide : f64 = 5.;

    let sixdaysago = now - chrono::Duration::days(6);

    if firstentry > sixdaysago
    {
        dbg!("HEY");
        last5ddtotdivide = last5ddtotdivide -
            firstentry.signed_duration_since(sixdaysago).num_days() as f64;
    }

    dbg!(firstentry);
    dbg!(last5ddtotdivide);

    // last1wk
    let last1wktotalsum : f64 = last1wktot.iter().map(|(val,_)| val).sum();
    let last1wknum = last1wktot.iter().map(|&(_, val)| val).max().unwrap();

    // last6wk
    let last6wktotalsum : f64 = last6wktot.iter().map(|(val,_)| val).sum();
    let last6wknum = last6wktot.iter().map(|&(_, val)| val).max().unwrap();

    // month
    let monthtotalsum : f64 = monthtot.iter().sum();
    let monthtotalavg : f64 = monthtotalsum /
        (relevantddcount_month_current(&firstentry_datetime(&db)?, &now)) as f64;

    println!("");
    println!("---------------------------------------------------------------");
    println!("Today:     {:6.2} (last 5 day avg: {:.2})",
              todaytotallsum, last5ddtotsum / last5ddtotdivide);
    println!("");
    println!("Current week");
    println!(" -> total  {:6.2}", weektotalsum);
    println!(" -> avg/d  {:6.2}", weektotalavg);
    println!("Last week");
    println!(" -> total  {:6.2}", last1wktotalsum);
    println!(" -> avg/d  {:6.2}", last1wktotalsum / last1wknum as f64);
    println!("Last {} weeks", last6wknum);
    println!(" -> tot/w  {:6.2}", last6wktotalsum / last6wknum as f64);
    println!(" -> avg/d  {:6.2}", last6wktotalsum / last6wknum as f64 / 7.);
	// println!("This week projected: {:6.2}", week_avg * 7.);
    println!("---------------------------------------------------------------");
    println!("This month total:    {:6.2}", monthtotalsum);
    println!("This month avg/day:  {:6.2}", monthtotalavg);
    println!("---------------------------------------------------------------");

    print!("Print detailed statistics per activity? (y/n): ");
    io::stdout().flush().unwrap();
   	let mut choice : String = Default::default();

    loop
    {
    	choice.clear();
        io::stdin().read_line(&mut choice).expect("Failed to read line");
        if ["n", "N"].contains(&choice.trim())
        { 
            println!(" - - - ");
            return Ok(());
        };
        if ["y", "Y"].contains(&choice.trim()) { break; };
    }

    for (index, item) in activitynames.iter().enumerate()
    {
        println!("---- Activity {}", item);

        let weektotalsum : f64 = week_tot[index];

        let weektotalavg = weektotalsum / daysrelevant as f64;
        let todaytotallsum : f64 = todaytot[index];
        let last5ddtotsum : f64 = last5ddtot[index];
        let (last1wktotalsum, last1wknum) = last1wktot[index];
        let (last6wktotallsum, last6wknum) = last6wktot[index];

        let alltime : f64 = db.query_row(
            &format!("SELECT hourstotal FROM {} WHERE id = ?",
                     crate::db::queries::SQL_TABLEN_ACT),
                     rusqlite::params![index+1], 
                     |row| row.get(0)).unwrap();

        println!("-----------------------------------------------------------");
        println!("Today:     {:6.2} (last 5 day avg: {:.2})",
        todaytotallsum, last5ddtotsum / 5.);
        println!("All time:  {:6.2}", alltime);
        println!("");
        println!("Current week");
        println!(" -> total  {:6.2}", weektotalsum);
        println!(" -> avg/d  {:6.2}", weektotalavg);
        println!("Last week");
        println!(" -> total  {:6.2}", last1wktotalsum);
        println!(" -> avg/d  {:6.2}", last1wktotalsum / last1wknum as f64);
        println!("Last {} weeks", last6wknum);
        println!(" -> tot/w  {:6.2}", last6wktotallsum / last6wknum as f64);
        println!(" -> avg/d  {:6.2}", 
                 last6wktotallsum / last6wknum as f64 / 7.);
        println!("-----------------------------------------------------------");
    }

    Ok(())
}

pub fn printstats_year(db : &Connection)
    -> Result<(), Box<dyn error::Error>>
{
    let now = chrono::Local::now();
    let now_year = now.year();

    // collect all yearly stats

    let mut statsvec : Vec<helpers::YearCounts> = Vec::new();
    let mut year = now_year;

    loop
    {
        if let Ok(stats) = helpers::
            retrieve_percentages_for_year(db, year, now)
        {
            statsvec.push(stats);
        }
        else
        { 
            break;
        }

        year -= 1;
    }

    if statsvec.is_empty()
    {
        println!("Could not retrieve yearly data");
        println!("Are you sure you have times in the db?");
        return Ok(());
    }

    // print current year and last year
    // print all time statistics for all past years

    let mut alltime = helpers::YearCounts::new();

    let mut count = 0;

    for s in statsvec
    {
        alltime = &alltime + &s;

        if count == 0 { println!("This year"); }
        if count == 1 { println!("Last year"); }
        if count >= 2 
        { 
            count += 1;
            continue; 
        }

        count += 1;
        s.printpercentages();
    }

    if count > 1
    {
        println!("ALL TIME: ");
        alltime.printpercentages();
    }

    println!("---------------------------------------------------------------");

    Ok(())
}

