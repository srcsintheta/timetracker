// module with logic shared across the crate's tests;
// originally the vision was shared access to an empty and initiliazed 
// w/ data db all tests could use; this came w/ too many headaches
// as it is now:
//  a) still a good place for shared test logic
//  b) makes sure tests operate on same db layout
//  (even though they all work on their own in_memory db)
//  (though small ones, should be fine within this scope)

/*
 * WARNING; BE AWARE
 * integral changes here could lead to all tests failing
 * tests assume this table to be here as is
 */

use rusqlite::Connection;
use chrono::{Datelike, TimeZone};
use crate::db::queries::*;

pub fn initialize_db(conn : &mut Connection) -> ()
{
    conn.execute(SQL_CREATE_ACT, ()).unwrap_or_else(|_| {
        panic!("Can't create table on in memory test db")
    });
    conn.execute(SQL_CREATE_HIS, ()).unwrap_or_else(|_| {
        panic!("Can't create table on in memory test db")
    });
}

pub fn populate_db_w_activities(conn : &mut Connection) -> ()
{
    let names = ["A", "B", "C", "D"];
    let date  = "2020-01-01"; // exact date of no particular signifiance

    for name in names
    {
        conn.execute(
            &format!(
                "INSERT INTO {} (name, added) VALUES (?1, ?2)", 
                SQL_TABLEN_ACT),
            rusqlite::params![name, date]
            ).unwrap_or_else(|_| panic!("Couldn't insert test activities"));
    }
}

// populates database with test data; tests assume this expected data to test
// their internal correctness; take that into account before changing anything
pub fn populate_db_w_data(conn : &mut Connection) -> ()
{
    // we start entries 2023-12-12, 21:00
    // every day we add 30 minutes per activity per day
    // every month we change starting time +2 hours
    // every month we change duration per activity +60 (+4 hours total)
    // we stop entries 2024-03-05, because
    // a) in this timeframe there's no DST changes (which are regional and
    // 	  therefore can't be easily universally tested for w/ DateTime<Local>)
    // b) 2024 is a leap year, we can obverse February working correctly

    let mut beg0 = 
        chrono::Local.with_ymd_and_hms(2023,12,12, 21,00,00).unwrap();
    let mut minutes = 30;
    let mut end1 = beg0 + chrono::Duration::minutes(minutes);
    let mut end2 = end1 + chrono::Duration::minutes(minutes);
    let mut end3 = end2 + chrono::Duration::minutes(minutes);
    let mut end4 = end3 + chrono::Duration::minutes(minutes);

    // 2023-12: working from 21 +  2 (to 23:00)
    // 2024-01: working from 23 +  6 (to 05:00)
    // 2024-02: working from 01 + 10 (to 11:00)
    // 2024-03: working from 03 + 14 (to 17:00) (2024-03-04 last day w/ entry)

    loop
    {
        crate::db::enter_into_db(conn, &beg0, &end1, 1).unwrap();
        crate::db::enter_into_db(conn, &end1, &end2, 2).unwrap();
        crate::db::enter_into_db(conn, &end2, &end3, 3).unwrap();
        crate::db::enter_into_db(conn, &end3, &end4, 4).unwrap();

        let month = beg0.month();
        beg0 += chrono::Duration::days(1);

        if beg0.month() != month
        { 
            minutes += 60;
            beg0 += chrono::Duration::hours(2);
        }
       
        end1 = beg0 + chrono::Duration::minutes(minutes);
        end2 = end1 + chrono::Duration::minutes(minutes);
        end3 = end2 + chrono::Duration::minutes(minutes);
        end4 = end3 + chrono::Duration::minutes(minutes);

        if beg0.day() == 05 && beg0.month() == 03 && beg0.year() == 2024
        	{ break; }
    }
}
