use regex::Regex;
use chrono::{Datelike, DateTime, Local, TimeZone};
use rusqlite::{Connection, params};

// helper function to clean a sql query
pub fn clean(input : String) -> String
{
    let s = input.replace("\n", " ").replace("\t", " ");
    let r = Regex::new(r"\s{2,}").unwrap();
    r.replace_all(&s, " ").to_string()
}

// helper function to round a float to six digits after decimal point
pub fn round(f : f64) -> f64
{
    (f * 1_000_000.).round() / 1_000_000.
}

// helper function to round a float to nine digits after decimal point
pub fn round9(f : f64) -> f64
{
    (f * 1_000_000_000.).round() / 1_000_000_000.
}

// helper function to compute number of days in a given month
pub fn days_in_month(year : i32, month : u32) -> u32
{
    chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or(chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
        .pred_opt().unwrap().day()
}

// helper function to compute number of days in a given year
pub fn days_in_year(year : i32) -> u32
{
    if chrono::NaiveDate::from_ymd_opt(year, 1, 1)
        .unwrap().leap_year()
    {
        366
    }
    else
    {
        365
    }
}

pub fn retrieve_first_entry_ymd(db : &mut Connection) -> DateTime<Local>
{
    let firstentry_str : String = db.query_row(
        &format!("SELECT MIN(date) FROM {}",
            super::queries::SQL_TABLEN_HIS),
        params![],
        |row| row.get(0)
        ).unwrap_or_else(|_| { "".to_string() });

    // process date string such we can construct a chrono::Local
    let parts : Vec<&str> = firstentry_str.split('-').collect();

    let yy : i32 = parts[0].parse().expect("Invalid year");
    let mm : u32 = parts[1].parse().expect("Invalid month");
    let dd : u32 = parts[2].parse().expect("Invalid day");

    chrono::Local.with_ymd_and_hms(yy, mm, dd, 01, 00, 00).unwrap()
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn clean_works()
    {
        let s = "Test\tstring\nfor cleaning\t".to_string();
        assert_eq!(clean(s), "Test string for cleaning ".to_string());
    }

    #[test]
    fn days_in_month_works()
    {
        assert_eq!(days_in_month(2024, 01), 31);
        assert_eq!(days_in_month(2024, 02), 29);
        assert_eq!(days_in_month(2025, 02), 28);
    }

    #[test]
    fn days_in_year_works()
    {
        assert_eq!(days_in_year(2023), 365);
        assert_eq!(days_in_year(2024), 366);
    }
}

