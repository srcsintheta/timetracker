pub const SQL_TABLEN_ACT : &str = "tt_activities";
// CAREFUL: other tables reference activities table BY NAME!
// PRIMARY implies NOT NULL and UNIQUE
pub const SQL_CREATE_ACT : &str =
"CREATE TABLE tt_activities (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL, 
    added TEXT NOT NULL, 
    hourstotal NUMERIC NOT NULL DEFAULT 0.0
    )";

pub const SQL_TABLEN_HIS : &str = "tt_history";
pub const SQL_CREATE_HIS : &str = 
"CREATE TABLE tt_history (
    id INTEGER NOT NULL, 
    year INTEGER NOT NULL, 
    month INTEGER NOT NULL, 
    day INTEGER NOT NULL, 
    isoweek INTEGER NOT NULL, 
    isoweekyear INTEGER NOT NULL,
    hoursonday NUMERIC NOT NULL DEFAULT 0.0, 
    date TEXT NOT NULL,
    FOREIGN KEY (id) REFERENCES tt_activities(id)
    )";

/*
 * tables `tt_statweekly`, `tt_statmonthly`, `tt_statyearly` once existed,
 * but have been removed; trivial to compute from `tt_history`;
 * not worth additional queries, creation, checking, entry/removal logic...
 */

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn creationquery_contains_tablename()
    {
        assert!(SQL_CREATE_ACT.to_string().contains(SQL_TABLEN_ACT));
        assert!(SQL_CREATE_HIS.to_string().contains(SQL_TABLEN_HIS));
    }
}
