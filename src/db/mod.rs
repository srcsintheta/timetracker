//! handles most db specific functionality
//! (initialization, integrity checking, ...)
//! stat functionality ousted to submodule stat

pub mod helpers;
pub mod queries;
pub mod stat;

use std::error;
use chrono::{DateTime, Datelike, Duration, Local};
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::Result;
use queries::*;
use helpers::*;

/// representing a row from Activities table;
/// only id, name, is_activated
#[derive(Debug, Clone)]
pub struct ActivitiesRow {
    pub id: i32,
    pub name: String,
}

/// representing a row from History table;
/// only id, date, hours
#[derive(Debug, Clone)]
pub struct HistoryRow {
    pub id: i32,
    pub date: String,
    pub hours: f64,
}

/// representing a row from StatWeekly table;
/// only id, wkno, hours
#[derive(Debug, Clone)]
pub struct StatWeeklyRow {
    pub id: i32,
    pub wkno: i32,
    pub hours: f64,
}

/// initialize a newly created db and populate with user data
/// after table creation hands user off to crate::conf() (see lib.rs)
pub fn init(db: &mut Connection) -> Result<()> {
    db.execute(SQL_CREATE_ACT, ())?;
    db.execute(SQL_CREATE_HIS, ())?;

    // initialization complete
    // send user off to configure the db (add activities and such)
    crate::conf(db)?;

    Ok(())
}

/// check existing db for integrity, conforming to expected layout
pub fn check(db : &Connection) -> Result<()> {
    // more extensive check
    // compare creation schema versus one from sqlite_master

    let mut letspanic = false;

    let mut stmt = db.prepare(
        "SELECT sql FROM sqlite_master WHERE type='table' AND name=?1",
    )?;

    let schema_act: String =
        stmt.query_row(params![SQL_TABLEN_ACT], |row| row.get(0))?;
    let schema_his: String =
        stmt.query_row(params![SQL_TABLEN_HIS], |row| row.get(0))?;

    println!(
        "Checking if needed tables exist and are d'accord w/ creation queries"
    );

    if !(clean(schema_act) == clean(SQL_CREATE_ACT.to_string())) {
        println!("table {} failed integrity check", SQL_TABLEN_ACT);
        letspanic = true;
    }
    if !(clean(schema_his) == clean(SQL_CREATE_HIS.to_string())) {
        println!("table {} failed integrity check", SQL_TABLEN_HIS);
        letspanic = true;
    }

    if !letspanic {
        println!("  Passed");
    } else {
        panic!("DB tables failed integrity check, something's off");
    }

    Ok(())
}

/// retrieve activities from activities table;
/// uses ActivitiesRow struct
pub fn get_activities(
    db			: &mut Connection,
    activated 	: bool,
) -> Result<Vec<ActivitiesRow>, Box<dyn error::Error>> {

	let mut stmt : rusqlite::Statement;

    if activated
    {
        stmt = db.prepare(
            &format!("SELECT id, name FROM {} 
                     WHERE id > 0 ORDER BY id ASC",
                     SQL_TABLEN_ACT)
            )?;
    }
    else
    {
        stmt = db.prepare(
            &format!("SELECT id, name FROM {}
                     WHERE id < 0 ORDER BY id DESC",
                     SQL_TABLEN_ACT)
            )?;
    }

    // create iterator
    let db_activities_data = stmt.query_map([], |row| {
        Ok(ActivitiesRow {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    // create data vector and use iterator to populate it
    let mut activities = Vec::new();
    for act in db_activities_data {
        activities.push(act?);
    }

    Ok(activities)
}

/// given a db and activity id retrieves the activitie's name
pub fn get_activityname_for_id(
    db: &Connection,
    actid: i32,
) -> Result<String, Box<dyn error::Error>> {
    let name: String = db
        .query_row(
            &format!("SELECT name FROM {} WHERE id = ?1", SQL_TABLEN_ACT),
            params![actid],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "".to_string());

    if name == "" {
        return Err("No such activity in db".into());
    }

    Ok(name)
}

/// make an entry into the db; handles midnight turnover, localtime updates, 
/// and invokes the entry functions for the stat tables; is also used for 
/// manual db entries;
pub fn enter_into_db(
    db: &mut Connection,
    dtbeg: &DateTime<Local>,
    dtend: &DateTime<Local>,
    actid: i32,
) -> Result<()> {
    let mut novalue = false;
    let mut ddchanged = false;
    let mut tillmidnight: f64 = 0.;
    let mut frommidnight: f64 = 0.;

    // signed_duration_since works with the internal UTC time of both
    // DateTime<Local> objects; it'll adjust for DST & timezone updates!
    let duration = dtend.signed_duration_since(dtbeg);
    let durationhours = duration.num_seconds() as f64 / 3600.;

    if durationhours >= 24.0
    {
        println!("Times of >= 24 hours aren't supported");
        println!("No time has been entered into the db");
        return Ok(());
    }

    let datebeg = dtbeg.format("%Y-%m-%d").to_string();
    let dateend = dtend.format("%Y-%m-%d").to_string();

    if dtbeg.date_naive() < dtend.date_naive() 
    {
        // important to test for `<` and not `!=` here
        // in strict theory dtend.date_naive() can be before dtbeg.date_naive()
        // (consider taking a flight slightly past midnight, flying eastward
        // such that you land "on the previous day")
        // in such a case we'll simply ignore the date change

        ddchanged = true;

        // we work w/ date_naive()! here which ignores localtime updates

        let nextmidnight = (dtbeg.naive_local() + Duration::days(1))
            .date()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let lastmidnight = dtend.date_naive().and_hms_opt(0, 0, 0).unwrap();

        tillmidnight =
            (nextmidnight - dtbeg.naive_local()).num_seconds() as f64 / 3600.;

        frommidnight =
            (dtend.naive_local() - lastmidnight).num_seconds() as f64 / 3600.;

        // if their sum is greater than previously computed duration
        // we adjust the end time to reflect the UTC offset change which occured

        if tillmidnight + frommidnight > durationhours {
            frommidnight =
                frommidnight - (tillmidnight + frommidnight - durationhours)
        }
    }

    // retrieve hours on day for activity
    // if day changed new day can be expected to not have an entry
    // so only beginning interests us here

    let mut hours_on_day: f64 = db
        .query_row(
            &format!(
                "SELECT hoursonday FROM {} WHERE date=?1 AND id=?2",
                SQL_TABLEN_HIS),
            params![datebeg, actid],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| {
            novalue = true;
            0.0
        });

    // also retrieve total_hours, since we update those

    let mut total_hours: f64 = db
        .query_row(
            &format!("SELECT hourstotal FROM {} WHERE id=?1", SQL_TABLEN_ACT),
            [actid],
            |row| row.get(0),
        )
        .unwrap_or(0.);

    // actually lets update total_hours right now
    // it's trivial and doesn't depend on whether we've worked past midnight

    total_hours += round(durationhours);
    total_hours = round(total_hours);

    db.execute(
        &format!("UPDATE {} SET hourstotal=?1 WHERE id=?2", SQL_TABLEN_ACT),
        params![total_hours, actid],
    )?;

    // prep our SQL queries

    let insertquery = clean(format!(
        "INSERT INTO {} 
            (id, year, month, day, isoweek, isoweekyear, hoursonday, date) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        SQL_TABLEN_HIS
    ));

    let updatequery = clean(format!(
        "UPDATE {} SET hoursonday=?1 WHERE id=?2 AND date=?3",
        SQL_TABLEN_HIS
    ));

    // let's first concern ourselve w/ what to enter for tpointstart.day

    if !ddchanged {
        hours_on_day += durationhours;
    } else {
        hours_on_day += tillmidnight;
    }

    hours_on_day = round(hours_on_day);

    if novalue {
        // insert
        db.execute(
            insertquery.as_str(),
            params![
                actid,
                dtbeg.year(),
                dtbeg.month(),
                dtbeg.day(),
                dtbeg.iso_week().week(),
                dtbeg.iso_week().year(),
                hours_on_day,
                datebeg,
            ],
        )
        .unwrap();
    } else {
        // update
        db.execute(updatequery.as_str(), params![hours_on_day, actid, datebeg])
            .unwrap();
    }

    if ddchanged {
        // we know there can't be an entry, just insert
        db.execute(
            insertquery.as_str(),
            params![
                actid,
                dtend.year(),
                dtend.month(),
                dtend.day(),
                dtend.iso_week().week(),
                dtend.iso_week().year(),
                round(frommidnight),
                dateend,
            ],
        )
        .unwrap();
    }

    Ok(())
}

/// remove an entry fully from db (history and stats tables)
pub fn remove_from_db(
    db: &mut Connection,
    date: &DateTime<Local>,
    actid: i32,
) -> Result<(), Box<dyn error::Error>> {
    assert!(actid > 0);

    // retrieve history entry
    // try deduction of hours from every table
    // if it doesn't lead to negative values apply removal and changes
    
    // retrieve hours
    let hours: f64 = db
        .query_row(
            format!(
                "SELECT hoursonday FROM {} WHERE id=?1 AND date=?2",
                SQL_TABLEN_HIS
            )
            .as_str(),
            params![actid, date.format("%Y-%m-%d").to_string()],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| -1.);

    if hours == -1. {
        return Err("There's no such entry in your history".into());
    }

    // try deduction of hours from every table
    // activities
    // statwk
    // statmm
    // statyy
    //
    // it should not be possible that an entry was entered correctly
    // but that its hour deduction leads to a negative value anywhere
    // this is why the checks are assert!()s

    // activities
    let hours_activities: f64 = db
        .query_row(
            &format!(
                "SELECT hourstotal FROM {} WHERE id=?1",
                SQL_TABLEN_ACT
            ),
            params![actid],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| 0.0);

    assert!(hours_activities - hours >= 0.);

    // apply changes
    //   remove from history table
    //   change activities

    let mut rowschanged;

    // remove from history table

    rowschanged = db
        .execute(
            &format!(
                "DELETE FROM {} WHERE id=?1 AND date=?2",
                SQL_TABLEN_HIS
            ),
            params![actid, date.format("%Y-%m-%d").to_string()],
        )
        .unwrap();

    assert!(rowschanged == 1);

    // change activities

    rowschanged = db
        .execute(
            &format!(
                "UPDATE {} SET hourstotal=?1 WHERE id=?2",
                SQL_TABLEN_ACT
            ),
            params![hours_activities - hours, actid],
        )
        .unwrap();

    assert!(rowschanged == 1);

    Ok(())
}

/// retrieve eight day history; used to prompt for data removal
pub fn retrieve_8day_history(
    db: &mut Connection,
) -> Result<Vec<HistoryRow>, Box<dyn error::Error>> {
    /*
        #[derive(Debug, Clone)]
        pub struct HistoryRow {
            pub id		: i32,
            pub date	: String,
            pub hours	: f64,
        }
    */

    let today = chrono::Local::now();
    let past8 = today - chrono::Duration::days(8);

    let mut stmt = db.prepare(
        &format!("SELECT * FROM {} WHERE date > ?1 ORDER BY date DESC",
        SQL_TABLEN_HIS)
    )?;

    // create iterator
    let db_history_data =
        stmt.query_map(params![past8.format("%Y-%m-%d").to_string()], |row| {
            Ok(HistoryRow {
                id: row.get(0)?,
                date: row.get(7)?,
                hours: row.get(6)?,
            })
        })?;

    // create data vector and use iterator to populate it
    let mut history = Vec::new();
    for his_entry in db_history_data {
        history.push(his_entry?);
    }

    Ok(history)
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test; // crate w/ shared test logic
    
    #[test]
    #[should_panic(expected = "failed integrity check")]
    fn extra_table_column_integrity_check()
    {
        let mut testdb = Connection::open_in_memory()
            .expect("Failed to open");

        test::initialize_db(&mut testdb);

        // add one extra column to history table
        testdb
            .execute(
                &format!(
                    "ALTER TABLE {} ADD COLUMN TEST INTEGER",
                    SQL_TABLEN_HIS),
                (),
            )
            .unwrap_or_else(|_| panic!("Couldn't add table column"));

        // integrity check should now panic
        let _ = check(&testdb);
    }

    #[test]
    fn retrieve_activities()
    {
        let mut testdb = Connection::open_in_memory().unwrap();
        test::initialize_db(&mut testdb);
        test::populate_db_w_activities(&mut testdb);
        let vec = get_activities(&mut testdb, true).unwrap();

        assert_eq!(4, vec.len());
        assert_eq!(vec[0].name, "A");
        assert_eq!(vec[1].name, "B");
        assert_eq!(vec[2].name, "C");
        assert_eq!(vec[3].name, "D");
    }

    
    #[test]
    // uses populated in memory db and tests expectations on tables
    fn entry_into_db_tables()
    {
        let mut db = Connection::open_in_memory().unwrap();
        test::initialize_db(&mut db);
        test::populate_db_w_activities(&mut db);
        test::populate_db_w_data(&mut db);
        let epsilon = 0.001;

        // lets retrieve some values from our tables and check correctness

        /*
         * activities table
         */

        let hours_total : f64 = db.query_row(
            &format!(
                "SELECT hourstotal FROM {} WHERE id=1",
                SQL_TABLEN_ACT), (),
                |row| row.get(0)).unwrap_or_else(|_| 0.0);

        // 2023-12: 20 * 02 = 040
        // 2024-01: 31 * 06 = 186
        // 2024-02: 28 * 10 = 280
        // 2024-03: 04 * 14 = 056
        
        assert!(hours_total > 0.);
        assert!((hours_total - (562. / 4.)).abs() <= epsilon);

        /*
         * singular entry
         */

        // in January we have activity 1 from 23 to 01:00
        // testing whether midnight turnover is correctly handled
        // for January 1st this should mean 1 hour
        // for January 2nd this should mean 2 hours

        let singularentry : f64 = db.query_row(
            &format!(
                "SELECT hoursonday FROM {} WHERE id=1 AND date='2024-01-01'",
                SQL_TABLEN_HIS), (),
                |row| row.get(0)).unwrap_or_else(|_| 0.0);

        assert!(singularentry > 0.);
        assert!((singularentry - 1.).abs() <= epsilon);

        let singularentry : f64 = db.query_row(
            &format!(
                "SELECT hoursonday FROM {} WHERE id=1 AND date='2024-01-02'",
                SQL_TABLEN_HIS), (),
                |row| row.get(0)).unwrap_or_else(|_| 0.0);

        assert!(singularentry > 0.);
        assert!((singularentry - 1.5).abs() <= epsilon);

        // writing generic test to check correct handling of localtime updates
        // is rather cumbersome
        // a) DateTime<Local>'s utc offset can't be changed manually
        // b) enter_into_db takes DateTime<Local> (rewrite too much work)
        // #[test]
        // fn check_utc_offset_handling()
        // therefore not written
    }

}
