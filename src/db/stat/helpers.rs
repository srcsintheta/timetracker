use core::ops::Add;
use crate::db::queries::*;
use std::error;
use rusqlite::{Connection, Error, params};
use chrono::{Datelike, Local, TimeZone, NaiveDate, Weekday};

fn max_iso_week(year: i32) -> u32
{
    let mut last_day = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
    while last_day.weekday() != Weekday::Thu
    {
        last_day = last_day.pred_opt().unwrap();
    }
    last_day.iso_week().week()
}

pub fn firstentry_datetime(db : &Connection)
    -> Result<chrono::DateTime<Local>, Box<dyn error::Error>>
{
	let firstyy : i32 = db.query_row(
        &format!("SELECT year  FROM {} ORDER BY date ASC LIMIT 1", 
                 SQL_TABLEN_HIS), [], |row| row.get(0))
        .unwrap_or(-1);

    if firstyy == -1 { return Err("No such entry".into()); }

	let firstmm : u32 = db.query_row(
        &format!("SELECT month FROM {} ORDER BY date ASC LIMIT 1", 
                 SQL_TABLEN_HIS), [], |row| row.get(0)).unwrap();
	let firstdd : u32 = db.query_row(
        &format!("SELECT day   FROM {} ORDER BY date ASC LIMIT 1", 
                 SQL_TABLEN_HIS), [], |row| row.get(0)).unwrap();

    Ok(chrono::Local
        .with_ymd_and_hms(firstyy, firstmm, firstdd, 0, 0, 0)
        .unwrap())
}

pub fn relevantddcount(
    firstentry  : &chrono::DateTime<Local>,
    year 		: i32,
    today		: &chrono::DateTime<Local>,
    ) -> i32
{
    let mut res = crate::db::helpers::days_in_year(year) as i32;

    if firstentry.year() == year
    {
        res -= firstentry.ordinal0() as i32;
    }
    if year == today.year()
    {
        res -= 
            crate::db::helpers::days_in_year(year) as i32
            - today.ordinal0() as i32;
    }

    res
}

pub fn relevantwkcount(
    firstentry	: &chrono::DateTime<Local>,
    year		: i32,
    today		: &chrono::DateTime<Local>,
    ) -> i32
{
    let mut res = max_iso_week(year) as i32; // therefore max
    
    if year == today.iso_week().year()
    {
        let to_subtract = max_iso_week(year) - today.iso_week().week0();
        res -= to_subtract as i32;
    }

    /*
     * if we're in the year of the first entry
     * disregard all weeks from 1 up to week included of first entry 
     */
    if year == firstentry.iso_week().year()
    {
        res = res - (firstentry.iso_week().week() as i32);
    }
    /* unless week started on Monday, then let's count it
     */
    if firstentry.weekday().number_from_monday() == 1
    {
        res += 1;
    }

    if res < 0 { return 0; }
    res
}


/* careful: includes today */
pub fn relevantddcount_month_current(
    firstentry  : &chrono::DateTime<Local>,
    today 		: &chrono::DateTime<Local>,
    ) -> i32
{
    let daysinmonth = crate::db::helpers::days_in_month(
        today.year(), today.month());

    let mut res = daysinmonth;

    if today.month() == firstentry.month() &&
        today.year() == firstentry.year()
        { 
            res -= firstentry.day0();
            res -= daysinmonth - today.day0();
        }

    res as i32
}

/// retrieve total of this week
/// (from today included up to last Monday included)
pub fn retrieve_total_this_week(
    db 	  : &Connection,
    today : chrono::DateTime<Local>,
    id	  : i32,
    )
    -> Result<f64, Error>
{
    let idstr = id.to_string();
    let mut values = Vec::new();
    let mut day = today - chrono::Duration::days(1);
    let isoweek = today.iso_week().week();
    if day.iso_week().week() != isoweek { return Ok(0.0); };

    loop
    {
        let datestring = day.format("%Y-%m-%d").to_string();

        let mut stmt = db.prepare(
            &format!("SELECT hoursonday FROM {} WHERE date = ?1 AND id = ?2",
                     SQL_TABLEN_HIS)
        )?;

        let hoursiter = stmt.query_map([datestring, idstr.clone()], |row| {
            row.get::<_, f64>(0)
        })?;

        for hour in hoursiter
        {
            values.push(hour?);
        }

        day = day - chrono::Duration::days(1);
        if day.iso_week().week() != isoweek { break; };
    }

    if values.is_empty()
    {
        return Ok(0.0);
    }
    else
    {
        Ok(values.iter().sum())
    }
}

/// retrieve total of this month
/// (from yesterday included back to first of month included)
pub fn retrieve_total_this_month(
    db 	  : &Connection,
    today : chrono::DateTime<Local>,
    id	  : i32,
    )
    -> Result<f64, Error>
{
    let idstr = id.to_string();
    let mut values = Vec::new();
    let mut day = today - chrono::Duration::days(1);
    let month = day.month();
    if day.month() != month { return Ok(0.0); };


    loop
    {
        let datestring = day.format("%Y-%m-%d").to_string();

        let mut stmt = db.prepare(
            &format!("SELECT hoursonday FROM {} WHERE date = ? AND id = ?", 
                     SQL_TABLEN_HIS)
        )?;

        let hoursiter = stmt.query_map([datestring, idstr.clone()], |row| {
            row.get::<_, f64>(0)
        })?;

        for hour in hoursiter
        {
            values.push(hour?);
        }

        day = day - chrono::Duration::days(1);
        if day.month() != month { break; };
    }

    if values.is_empty()
    {
        return Ok(0.0);
    }
    else
    {
        Ok(values.iter().sum())
    }
}

/// retrieve total of the last x days
/// from yesterday included back to 10 days prior included
pub fn retrieve_total_last_x_days(
    db    : &Connection,
    x     : i64,
    start : chrono::DateTime<Local>,
    id	  : i32,
    )
    -> Result<f64, Error>
{
    assert!(x >= 0);

    let idstr = id.to_string();

    let mut values = Vec::new();

    for i in 0..x
    {
        let prev = start - chrono::Duration::days(i + 1);
        let prev_datestring = prev.format("%Y-%m-%d").to_string();

        let mut stmt = db.prepare(
            &format!("SELECT hoursonday FROM {} WHERE date = ?1 AND id = ?2",
                     SQL_TABLEN_HIS)
        )?;

        let hoursiter = stmt.query_map([prev_datestring, idstr.clone()], |row| {
            row.get::<_, f64>(0)
        })?;

        for hour in hoursiter
        {
            values.push(hour?);
        }
    }

    if values.is_empty()
    {
        return Ok(0.0);
    }
    else
    {
        Ok(values.iter().sum())
    }
}

pub fn retrieve_total_last_x_weeks(
    db : &Connection,
    x  : i64,
    dt : chrono::DateTime<Local>,
    id : i32,
    )
    -> Result<(f64, i64), Box<dyn error::Error>>
{
    assert!(x >= 0); 

    let mut values = Vec::new();

    let firstentry = firstentry_datetime(&db)?;
    let mut num_weeks = 0;

    for i in 0..x
    {
        let prev_week  = dt  - chrono::Duration::weeks(i + 1);
        /*
         * first entry week is disregarded if it hasn't started on Monday
         */
		if prev_week.iso_week().week() == firstentry.iso_week().week() &&
            prev_week.iso_week().year() == firstentry.iso_week().year() &&
                firstentry.weekday().number_from_monday() != 1
        {
            break;
        }

        let isowk = prev_week.iso_week().week();
        let year  = prev_week.iso_week().year(); // ! DO NOT USE .year()

        let mut stmt = db.prepare(
            &format!("SELECT hoursonday FROM {}
                    WHERE isoweek=? AND isoweekyear=? AND id=?", SQL_TABLEN_HIS)
        )?;

        let hoursiter = stmt.query_map([isowk, year as u32, id as u32], |row| {
            row.get::<_, f64>(0)
        })?;

        for hour in hoursiter
        {
            values.push(hour?);
        }

        num_weeks = i + 1;
    }

    if values.is_empty()
    {
        return Ok((0.0, num_weeks));
    }
    else
    {
        Ok((values.iter().sum(), num_weeks))
    }
}
    
pub fn retrieve_total_today(
    db : &Connection,
    dt : chrono::DateTime<Local>,
    id : i32
    )
    -> Result<f64, Error>
{
    let idstr = id.to_string();
    let mut values = Vec::new();
    let today = dt.format("%Y-%m-%d").to_string();

    let mut stmt = db.prepare(
        &format!("SELECT hoursonday FROM {} 
                 WHERE date = ?1 AND id = ?2",
                 SQL_TABLEN_HIS)
        )?;

    let hours_iter = stmt.query_map([today, idstr.clone()], |row| {
        row.get::<_, f64>(0)
    })?;

    for hour in hours_iter
    {
        values.push(hour?);
    }

    if values.is_empty()
    {
        return Ok(0.0);
    }
    else
    {
        Ok(values.iter().sum())
    }
}

#[derive(Debug)]
pub struct YearCounts {
    pub dd_00_hrs	 : i32,
    pub dd_04_hrspls : i32,
    pub dd_08_hrspls : i32,
    pub dd_10_hrspls : i32,
    pub dd_12_hrspls : i32,
    pub dd_14_hrspls : i32,
    pub dds_relevant : i32,
    pub wk_00_hrs    : i32,
    pub wk_10_hrspls : i32,
    pub wk_20_hrspls : i32,
    pub wk_30_hrspls : i32,
    pub wk_40_hrspls : i32,
    pub wk_50_hrspls : i32,
    pub wk_60_hrspls : i32,
    pub wk_70_hrspls : i32,
    pub wk_80_hrspls : i32,
    pub wk_90_hrspls : i32,
    pub wks_relevant : i32,

}

impl YearCounts
{
    pub fn new() -> YearCounts
    {
        YearCounts {
        dd_00_hrs	 : 0,
        dd_04_hrspls : 0,
        dd_08_hrspls : 0,
        dd_10_hrspls : 0,
        dd_12_hrspls : 0,
        dd_14_hrspls : 0,
        dds_relevant : 0,
        wk_00_hrs	 : 0,
        wk_10_hrspls : 0,
        wk_20_hrspls : 0,
        wk_30_hrspls : 0,
        wk_40_hrspls : 0,
        wk_50_hrspls : 0,
        wk_60_hrspls : 0,
        wk_70_hrspls : 0,
        wk_80_hrspls : 0,
        wk_90_hrspls : 0,
        wks_relevant : 0,
        }
    }

    pub fn printpercentages(self)
    {
        println!("  % of days  w/  0 hrs : {:6.2}", (
                self.dd_00_hrs as f64 / 
                self.dds_relevant as f64) * 100.0 );
        println!("  % of days  w/  4 hrs+: {:6.2}", (
                self.dd_04_hrspls as f64 /
                self.dds_relevant as f64) * 100.0 );
        println!("  % of days  w/  8 hrs+: {:6.2}", (
                self.dd_08_hrspls as f64 /
                self.dds_relevant as f64) * 100.0 );
        println!("  % of days  w/ 10 hrs+: {:6.2}", (
                self.dd_10_hrspls as f64 /
                self.dds_relevant as f64) * 100.0);
        println!("  % of days  w/ 12 hrs+: {:6.2}", (
                self.dd_12_hrspls as f64 / 
                self.dds_relevant as f64) * 100.0);
        /*
        println!("  % of days  w/ 14 hrs+: {:6.2}", (
                self.dd_14_hrspls as f64 / 
                self.dds_relevant as f64) * 100.0);
         */
        println!("  % of weeks w/  0 hrs : {:6.2}", (
                self.wk_00_hrs as f64	  / 
                self.wks_relevant as f64) * 100.0);
        /*
        println!("  % of weeks w/ 10 hrs+: {:6.2}", (
                self.wk_10_hrspls as f64 / 
                self.wks_relevant as f64) * 100.0);
         */
        println!("  % of weeks w/ 20 hrs+: {:6.2}", (
                self.wk_20_hrspls as f64 / 
                self.wks_relevant as f64) * 100.0);
        /*
        println!("  % of weeks w/ 30 hrs+: {:6.2}", (
                self.wk_30_hrspls as f64 / 
                self.wks_relevant as f64) * 100.0);
         */
        println!("  % of weeks w/ 40 hrs+: {:6.2}", (
                self.wk_40_hrspls as f64 / 
                self.wks_relevant as f64) * 100.0);
        println!("  % of weeks w/ 50 hrs+: {:6.2}", (
                self.wk_50_hrspls as f64 / 
                self.wks_relevant as f64) * 100.0);
        println!("  % of weeks w/ 60 hrs+: {:6.2}", (
                self.wk_60_hrspls as f64 / 
                self.wks_relevant as f64) * 100.0);
        println!("  % of weeks w/ 70 hrs+: {:6.2}", (
                self.wk_70_hrspls as f64 / 
                self.wks_relevant as f64) * 100.0);
        println!("  % of weeks w/ 80 hrs+: {:6.2}", (
                self.wk_80_hrspls as f64 / 
                self.wks_relevant as f64) * 100.0);
        /*
        println!("  % of weeks w/ 90 hrs+: {:6.2}", (
                self.wk_90_hrspls as f64 / 
                self.wks_relevant as f64) * 100.0);
         */

    }
}

impl<'a> Add for &'a YearCounts
{
    type Output = YearCounts;

    fn add(self, other: &'a YearCounts) -> YearCounts
    {
        YearCounts {
            dd_00_hrs: 	  self.dd_00_hrs    + other.dd_00_hrs,
            dd_04_hrspls: self.dd_04_hrspls + other.dd_04_hrspls,
            dd_08_hrspls: self.dd_08_hrspls + other.dd_08_hrspls,
            dd_10_hrspls: self.dd_10_hrspls + other.dd_10_hrspls,
            dd_12_hrspls: self.dd_12_hrspls + other.dd_12_hrspls,
            dd_14_hrspls: self.dd_14_hrspls + other.dd_14_hrspls,
            dds_relevant: self.dds_relevant + other.dds_relevant,
            wk_00_hrs	: self.wk_00_hrs	+ other.wk_00_hrs,
            wk_10_hrspls: self.wk_10_hrspls	+ other.wk_10_hrspls,
            wk_20_hrspls: self.wk_20_hrspls	+ other.wk_20_hrspls,
            wk_30_hrspls: self.wk_30_hrspls	+ other.wk_30_hrspls,
            wk_40_hrspls: self.wk_40_hrspls	+ other.wk_40_hrspls,
            wk_50_hrspls: self.wk_50_hrspls	+ other.wk_50_hrspls,
            wk_60_hrspls: self.wk_60_hrspls	+ other.wk_60_hrspls,
            wk_70_hrspls: self.wk_70_hrspls	+ other.wk_70_hrspls,
            wk_80_hrspls: self.wk_80_hrspls	+ other.wk_80_hrspls,
            wk_90_hrspls: self.wk_90_hrspls	+ other.wk_90_hrspls,
            wks_relevant: self.wks_relevant + other.wks_relevant,
        }

    }
}

#[derive(Debug)]
pub struct HistoryRowISO8601 {
    pub id	    : i32,
    pub date    : String,
    pub hours   : f64,
    pub isowk   : u32,
    pub isowkyy : i32,
    pub year	: i32,
}

pub fn retrieve_percentages_for_year(
    db    : &Connection,
    year  : i32,
    today : chrono::DateTime<Local>,
    )
    -> Result<YearCounts, Box<dyn error::Error>>
{
    let mut stats = YearCounts::new();

    // retrieve year,month,day of first entry (important to adjust statistics)

    let firstentry = firstentry_datetime(&db)?;
    let firstyy = firstentry.year();
    if year < firstyy { return Err("Out of scope".into()); }

    let relevantddcount = relevantddcount(&firstentry, year, &today);

    /*
     * COMPILE YEARLY STATS
     */

    /*
     * Retrieve data for a calendar year, store into a Vec<HistoryRow>
     */

    let mut stmt = db.prepare(
        &format!("SELECT * FROM {} WHERE year=?1 ORDER BY date ASC",
                 SQL_TABLEN_HIS)
        )?;

    let data_iter = stmt.query_map(
        params![year], |row|
        {
            Ok(crate::db::HistoryRow {
                id:		row.get(0)?,
                date:	row.get(7)?,
                hours:	row.get(6)?,
            })
        })?;

    let mut historyvec = Vec::new();

    for row in data_iter
    {
        historyvec.push(row?);
    }

    // STEP 2: let's deal with the daily percentages
    //   let's iterate over data
    //   collect all same days in a vector
    //   and count num of days that fulfill certain criteria

    /*
     * Iterate over vector
     * collect values for every singular day
     * count singular days fulfilling specific criteria
     */

    let mut days_with_entries = 0;
    let mut num_days_04_hours = 0;
    let mut num_days_08_hours = 0;
    let mut num_days_10_hours = 0;
    let mut num_days_12_hours = 0;
    let mut num_days_14_hours = 0;

	let mut iter = historyvec.iter().peekable();

    while let Some(entry) = iter.next()
    {
        let current_date = entry.date.clone();
        let mut current_date_entries = vec![entry];

        while let Some(&next) = iter.peek()
        {
            if next.date == current_date
            {
                current_date_entries.push(next);
                iter.next(); // advances iterator
            }
            else
            {
                break;
            }
        }

        // exclude today from statistics
        if entry.date == today.date_naive().format("%Y-%m-%d").to_string()
        {
            break;
        }

        days_with_entries += 1;

        let mut hoursum : f64 = 0.;

        // current_date_entries has values of one specific date collected now
        // calculate total hour sum for any given day

        for entry in current_date_entries
        {
            hoursum += entry.hours;
        }

        // based on this value count up criteria counters

        if hoursum >= 04. { num_days_04_hours += 1; }
        if hoursum >= 08. { num_days_08_hours += 1; }
        if hoursum >= 10. { num_days_10_hours += 1; }
        if hoursum >= 12. { num_days_12_hours += 1; }
        if hoursum >= 14. { num_days_14_hours += 1; }
    }

    let num_days_00_hours = (relevantddcount - days_with_entries).abs();

    stats.dd_00_hrs    = num_days_00_hours;
    stats.dd_04_hrspls = num_days_04_hours;
    stats.dd_08_hrspls = num_days_08_hours;
    stats.dd_10_hrspls = num_days_10_hours;
    stats.dd_12_hrspls = num_days_12_hours;
    stats.dd_14_hrspls = num_days_14_hours;
    stats.dds_relevant = relevantddcount;

    /*
     * COMPILE WEEKLY STATS
     */

    /*
     * Retrieve data for a ISOweekyear, store into a Vec<HistoryRowISO8601>
     */

    let mut stmt = db.prepare(
        &format!("SELECT * FROM {} WHERE isoweekyear=?1 ORDER BY date ASC",
                 SQL_TABLEN_HIS)
        )?;

    let data_iter = stmt.query_map(
        params![year], |row|
        {
            Ok(HistoryRowISO8601 {
                id:		row.get(0)?,
                date:	row.get(7)?,
                hours:	row.get(6)?,
                isowk:  row.get(4)?,
                isowkyy:row.get(5)?,
                year:	row.get(1)?,
            })
        })?;

    let mut historyvec = Vec::new();

    for row in data_iter
    {
        historyvec.push(row?);
    }

    // STEP 2: let's deal with the weekly percentages
    //   let's iterate over data
    //   collect all same weeks in a vector
    //   and count num of weeks that fulfill certain criteria

    /*
     * Iterate over vector
     * collect values for every singular day
     * count singular days fulfilling specific criteria
     */

    let mut num_weeks_taken_into_account = 0;
    let mut num_weeks_10_hours = 0;
    let mut num_weeks_20_hours = 0;
    let mut num_weeks_30_hours = 0;
    let mut num_weeks_40_hours = 0;
    let mut num_weeks_50_hours = 0;
    let mut num_weeks_60_hours = 0;
    let mut num_weeks_70_hours = 0;
    let mut num_weeks_80_hours = 0;
    let mut num_weeks_90_hours = 0;

	let mut iter = historyvec.iter().peekable();
    
    while let Some(entry) = iter.next()
    {
        let current_week = entry.isowk;
        let mut current_week_entries = vec![entry];

        /* 
         * two special scenarios though
         * a) we disregard very first week ever if it's partial
         * b) we disregard the last week if it's current
         *
         */
        /* 
         * a)
         *
         */
        if entry.isowkyy == firstentry.iso_week().year() &&
            entry.isowk == firstentry.iso_week().week() &&
                firstentry.weekday().number_from_monday() != 1
        {
                continue;
        }

        /* 
         * b)
         *
         */
        if entry.isowkyy == today.iso_week().year() &&
            entry.isowk == today.iso_week().week() 
        {
                break;
        }

        /*
         * rest as per usual
         *
         */

        while let Some(&next) = iter.peek()
        {
            if next.isowk == current_week
            {
                current_week_entries.push(next);
                iter.next(); // advances iterator
            }
            else
            {
                break;
            }
        }

        num_weeks_taken_into_account += 1;

        let mut hoursum : f64 = 0.;

        // current_week_entries has values of one specific week collected now
        // calculate total hour sum for any given week

        for entry in current_week_entries
        {
            hoursum += entry.hours;
        }

        // based on this value count up criteria counters

        if hoursum >= 10. { num_weeks_10_hours += 1; };
        if hoursum >= 20. { num_weeks_20_hours += 1; };
        if hoursum >= 30. { num_weeks_30_hours += 1; };
        if hoursum >= 40. { num_weeks_40_hours += 1; };
        if hoursum >= 50. { num_weeks_50_hours += 1; };
        if hoursum >= 60. { num_weeks_60_hours += 1; };
        if hoursum >= 70. { num_weeks_70_hours += 1; };
        if hoursum >= 80. { num_weeks_80_hours += 1; };
        if hoursum >= 90. { num_weeks_90_hours += 1; };
    }

    /*
     * RELEVANT WEEKCOUNT
     * remember, we're retrieving values for the iso_week().year() to begin w/
     */

    let relevantwkcount = relevantwkcount(&firstentry, year, &today);


    let num_weeks_00_hours = relevantwkcount - num_weeks_taken_into_account;

    /*
     * Fill up stats
     */

    stats.wk_00_hrs    = num_weeks_00_hours;
    stats.wk_10_hrspls = num_weeks_10_hours;
    stats.wk_20_hrspls = num_weeks_20_hours;
    stats.wk_30_hrspls = num_weeks_30_hours;
    stats.wk_40_hrspls = num_weeks_40_hours;
    stats.wk_50_hrspls = num_weeks_50_hours;
    stats.wk_60_hrspls = num_weeks_60_hours;
    stats.wk_70_hrspls = num_weeks_70_hours;
    stats.wk_80_hrspls = num_weeks_80_hours;
    stats.wk_90_hrspls = num_weeks_90_hours;
    stats.wks_relevant = relevantwkcount;

    Ok(stats)
}

/// testing submodule
/// to make sense of any of these tests you should have a calendar w/ iso weeks
/// nearby, and have src/test.rs open, to inspect the db data as well
#[cfg(test)]
mod tests
{
    use super::*;
    use chrono::TimeZone;
    use crate::test;

    #[test]
    fn all_small_helpers()
    {
        let mut testdb = Connection::open_in_memory().unwrap();
        test::initialize_db(&mut testdb);
        test::populate_db_w_activities(&mut testdb);
        test::populate_db_w_data(&mut testdb);

        let epsilon = 0.001;

        /* NOTE!!!
         * for most functions the time of our DateTime<Local> doesn't matter
         * retrieval is purely based on the date
         * NOTE!!! id passed has an influence (midnight turnover)
         */

        /* just have a look at the testdb layout used (src/test.rs)
         * and have a calendar handy, if you want to make sense of these
         */

        /*
         * retrieve_total_this_week
         */

		let dt = chrono::Local.with_ymd_and_hms(2024,01,04,03,00,00).unwrap();	
        let rttw = retrieve_total_this_week(&testdb, dt, 1).unwrap();
        assert!((rttw - (1.5 + 1.5 + 1.)).abs() <= epsilon);

        /*
         * retrieve_total_this_month
         */

        let rttm = retrieve_total_this_month(&testdb, dt, 1).unwrap();
        assert!((rttm - (1.5 + 1.5 + 1.)).abs() <= epsilon);
        let dt = chrono::Local.with_ymd_and_hms(2024,02,10,03,00,00).unwrap();
        let rttm = retrieve_total_this_month(&testdb, dt, 2).unwrap();
        assert!((rttm - (1.5 + 8. * 2.5)).abs() <= epsilon);

        /*
         * retrieve_total_last_x_days
         */

        let rtlxd = retrieve_total_last_x_days(&testdb, 9, dt, 2).unwrap();
        assert!((rtlxd - (1.5 + 8. * 2.5)).abs() <= epsilon);
        let dt = chrono::Local.with_ymd_and_hms(2025,02,10,03,00,00).unwrap();
        let rtlxd = retrieve_total_last_x_days(&testdb, 100, dt, 3).unwrap();
        assert!((rtlxd - 0.0).abs() <= epsilon);

        /*
         * retrieve_total_last_x_weeks
         */

        let (rtlxw,_) = retrieve_total_last_x_weeks(&testdb, 10, dt, 3)
            .unwrap();
        assert!((rtlxw - 0.0).abs() <= epsilon);
        let dt = chrono::Local.with_ymd_and_hms(2024,01,07,03,00,00).unwrap();
        let (rtlxw,_) = retrieve_total_last_x_weeks(&testdb, 10, dt, 4)
            .unwrap();
        assert!((rtlxw - (14. * 0.5)).abs() <= epsilon);

        /*
         * retrieve_total_today
         */

        let rtt = retrieve_total_today(&testdb, dt, 1).unwrap();
        assert!((rtt - 1.5).abs() <= epsilon);
        let dt = chrono::Local.with_ymd_and_hms(2024,02,01,03,00,00).unwrap();
        let rtt = retrieve_total_today(&testdb, dt, 3).unwrap();
        assert!((rtt - 1.5).abs() <= epsilon);
    }

    #[test]
    fn percentage_retrieval()
    {
        let mut testdb = Connection::open_in_memory().unwrap();
        test::initialize_db(&mut testdb);
        test::populate_db_w_activities(&mut testdb);
        test::populate_db_w_data(&mut testdb);
        let epsilon = 0.001;

        let now  = chrono::Local.with_ymd_and_hms(2024,03,20,0,0,0).unwrap();

        /* 2024-01: 31 days
         * 2024-02: 29 days
         * 2024-03: 04 days (2024-03-04 has last entry)
         * total  : 64 days
         */
 
        /*
         * use rttm to calculate days of months
         * retest that function since we're at it
         */

        let jan31 = chrono::Local.with_ymd_and_hms(2024,01,31,0,0,0).unwrap();
        let feb29 = chrono::Local.with_ymd_and_hms(2024,02,29,0,0,0).unwrap();
        let mar04 = chrono::Local.with_ymd_and_hms(2024,03,04,0,0,0).unwrap();

        // NOTE! ids have an effect, midnight turnover etc
        let jandays = retrieve_total_this_month(&testdb, jan31, 1).unwrap();
        let febdays = retrieve_total_this_month(&testdb, feb29, 2).unwrap();
        let mardays = retrieve_total_this_month(&testdb, mar04, 4).unwrap();

        // NOTE: (total excludes `today` of Local datetime object
        assert!((jandays - (1. + 29. * 1.5 )).abs() <= epsilon);
        assert!((febdays - (1.5 + 27. * 2.5)).abs() <= epsilon);
        assert!((mardays - (3. * 3.5       )).abs() <= epsilon);

        /*
         * 2024
         *
         */

        let yc24 = retrieve_percentages_for_year(&testdb, 2024, now).unwrap();

        /*
         * day values of 2024
         */

        // dd_00_hrs: days from 2024-03-05 till 2024-03-19 (both included)
        // (since chrono::Local passed to rpfy is that date)
        assert_eq!(yc24.dd_00_hrs   , 15);
        assert_eq!(yc24.dd_04_hrspls, 64 - 1);
        assert_eq!(yc24.dd_08_hrspls, 64 - 1 - 30 - 1);
        assert_eq!(yc24.dd_10_hrspls, 64 - 1 - 30 - 1);
        assert_eq!(yc24.dd_12_hrspls, 64 - 1 - 30 - 1 - 28);
        assert_eq!(yc24.dd_14_hrspls, 64 - 1 - 30 - 1 - 28);
        assert_eq!(yc24.dds_relevant, 64 + 15);

        /*
         * week values of 2024
         */

        assert_eq!(yc24.wks_relevant, 11); // used in subsequent calculations

        /*
         * last entry: 2024-03-04, now is 2024-03-20
         * so entries 05 to 19 w/ no values (this includes one fully empty week)
         */
        assert_eq!(yc24.wk_00_hrs,    1);
        // all relevant weeks minus the one w/ no entries
        assert_eq!(yc24.wk_10_hrspls, (11 - 1));
        // minus last week w/ only one entry (2024-03-04 is Monday w/ 14 hours)
        assert_eq!(yc24.wk_20_hrspls, (11 - 1 - 1));
        assert_eq!(yc24.wk_30_hrspls, (11 - 1 - 1));
        // minus first january week
        assert_eq!(yc24.wk_40_hrspls, 11 - 1 - 1 - 1);
        // minus complete full january weeks
        assert_eq!(yc24.wk_50_hrspls, 11 - 1 - 1 - 1 - 3);
        // minus januar/february spill over week
        assert_eq!(yc24.wk_60_hrspls, 11 - 1 - 1 - 1 - 3 - 1);
        assert_eq!(yc24.wk_70_hrspls, 11 - 1 - 1 - 1 - 3 - 1);
        // minutes all full february weeks
        assert_eq!(yc24.wk_80_hrspls, 11 - 1 - 1 - 1 - 3 - 1 - 3);
        // minus february/march spillover week, should be at 0 now
        assert_eq!(yc24.wk_90_hrspls, 11 - 1 - 1 - 1 - 3 - 1 - 3 - 1);

        /*
         * 2023
         *
         */

        let yc23 = retrieve_percentages_for_year(&testdb, 2023, now).unwrap();

        /*
         * day values of 2023
         */

        /* 2023 first year db was used
         * first day: 2023-12-12, so 20 entries in total
         * all days have 2 hours per day, testing day values a bit pointless
         */

        assert_eq!(yc23.dd_00_hrs   , 0);
        assert_eq!(yc23.dd_04_hrspls, 0);
        assert_eq!(yc23.dd_08_hrspls, 0);
        assert_eq!(yc23.dd_10_hrspls, 0);
        assert_eq!(yc23.dd_12_hrspls, 0);
        assert_eq!(yc23.dd_14_hrspls, 0);
        assert_eq!(yc23.dds_relevant, 20);

        /*
         * week values of 2023
         */

        // first week not considered, iso weeks 51 52 are relevant
        assert_eq!(yc23.wks_relevant, 2); // used in subsequent calculations

        assert_eq!(yc23.wk_00_hrs,    0);
        assert_eq!(yc23.wk_10_hrspls, 2);
        assert_eq!(yc23.wk_20_hrspls, 0);
        assert_eq!(yc23.wk_30_hrspls, 0);
        assert_eq!(yc23.wk_40_hrspls, 0);
        assert_eq!(yc23.wk_50_hrspls, 0);
        assert_eq!(yc23.wk_60_hrspls, 0);
        assert_eq!(yc23.wk_70_hrspls, 0);
        assert_eq!(yc23.wk_80_hrspls, 0);
        assert_eq!(yc23.wk_90_hrspls, 0);

        /*
         * TEST EXTRA: PROPER RELEVANT WEEK / DAYS HANDLING
         * 2023 2024 is an unusual test
         * a) 2023 wk 52 was a full week (monday to sunday), therefore
         * b) 2024 wk 01 was a full week (monday to sunday)
         * let's check if more complicated configurations are properly handled
         *
         * this "is it really needed" test actually caught a bug, hooray
         */

        /*
         * Test scenario: end of year is iso week of next year
         */

        let mut testdb = Connection::open_in_memory().unwrap();
        test::initialize_db(&mut testdb);
        test::populate_db_w_activities(&mut testdb);

        let firstentry_beg = chrono::Local.with_ymd_and_hms(
            2024, 12, 30, 0, 0, 0).unwrap();
        // iso week 1 of the year 2025, and a Monday
        let firstentry_end = firstentry_beg + chrono::Duration::hours(5);

        crate::db::enter_into_db(
            &mut testdb,
            &firstentry_beg,
            &firstentry_end,
            1).unwrap();

        let today = chrono::Local.with_ymd_and_hms(
            2025, 01, 18, 0, 0, 0).unwrap();

        let s = retrieve_percentages_for_year(
            &testdb,
            2025,
            today).unwrap();

        // since first entry is monday, the week is expected to be counted
        // even though it's calendar year 2024, it's iso week year 2025
        // 2025-01-18 is Saturday, this week is disregarded
        
        assert_eq!(s.wks_relevant,  2); // iso weeks in iso week year
        assert_eq!(s.wk_00_hrs,     1);

        assert_eq!(s.dds_relevant, 17); // actual days in calendar year
        assert_eq!(s.dd_00_hrs,    17); // actual calendar year!
        assert_eq!(s.dd_04_hrspls, 00); // actual calendar year!

        /* subsequent test removed, I think we have enough as is
         * and there isn't much more to test really
         */
    }

}
