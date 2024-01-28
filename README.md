# timetracker

## Description

Command line tool to track time spent on user-defined activities. Can record a
time via a timer, manual entry, and also has extensive capability showing
statistics. Uses a sqlite3 db in back to save activities, and all hours per
activity per day. Written and tested on 64-bit Linux only, but could work on
other systems (without actual testing I'm not confident enough to say that it
"should work", albeit I hope it does).

Testers may of course report their findings.

## Development history

* version 0.1.0: alpha version

## Demo

Preamble

* I will not update these demo screens every single time I make the tiniest
  change in the text interface. This example run should stay relevant in any
  case and is on point as of version 0.1.0 of course, but don't lynch me if by
  the time you run the real application some minor things are different
* `/* */` is the comment style I use in the demo to denote having skipped
  something, or hinting at something instead of printing it on screen to avoid
  needless repitition
* stat numbers are made up in this demo and for illustration's sake

### First start, db initialization

```
/* compilation output
 */

db file doesn't exist, creating: "/home/sr/.config/timetracker/productivity.db"
Productivity tracker
Version : 0.1.0
Database used: "/home/sr/.config/timetracker/productivity.db"
initializing db w/ needed tables
Options:

  (a)dd new
  (d)eactivate
  (r)eactivate
  (q)uit (back to main menu)

Your option:
```

At first we add new activities

```
Your option: a
Enter activity name: MainJob
Options:

  (a)dd new
  (d)eactivate
  (r)eactivate
  (q)uit (back to main menu)

Your option: a
Enter activity name: GigHomepageForSusie /* can contain spaces, np */
Options:

  (a)dd new
  (d)eactivate
  (r)eactivate
  (q)uit (back to main menu)

/* Add as many activities as you like
/* ...
 */

Your option: q
```

Since you can easily deactivate activities at any point later, it's on you
whether you want to create some solid activities that you plan to use long-term
for tracking your work (MainJob, SecondJob, Programming, ...), or create lots
of temporary ones like a temporary gig for someone (or do both). Adding a new
activity for projects, gigs, etc could provide helpful data on later
evaluations, billing clients etc...

### Main Menu

```
-----------------
--- Main Menu ---
-----------------
Available options

  1) track
  2) manual entry
  3) delete entry

  4) stats
  5) stats (yearly)

  6) configuration of activities
  7) exit

Your option:
```

### 1) track

```
/* main menu */

Your option: 1

---------------------------------------------------------------
ID	Name
1	MainJob
2	GigHomepageForSusie
---------------------------------------------------------------
Enter one of the listed activity IDs
  'q' to go back to main

Your input: 1
Press Enter to switch between work/break
Press q-Enter to end
Started work timer
  00:00:08
Work time thus far: 00:00:08
Started break timer
q 00:00:02
Break time thus far: 00:00:02

Total worked:	00:00:08
Total paused:	00:00:02
Pause percentage: 0.25%

/* main menu */
```

### 2) manual entry

If you want to use the application as a full tracker, you might of course want
to enter work time from you main job at the end of the day, et cetera.

```
/* main menu */

Your option: 2

For which activity do you want to add a time?
---------------------------------------------------------------
ID	Name
1	MainJob
2	GigHomepageForSusie
---------------------------------------------------------------
Enter one of the listed activity IDs
  'q' to go back to main

Your input: 1
For which day do you want to enter a time:
  1) Today
  2) Yesterday
  3) Specify date manually
  ('q' to go back to main)
```

The scenario of entering a time for `1) Today` or `2) Yesterday` is the most
common one for me. Providing those as automatic options is also useful, since
you don't have to type out dates frequently and there's less chance of user
error.

```
/* for which day you want to enter a time menu from screen before */

Your input: 1
Enter duration (hours and minutes):
  Hours   (0-23): 8
  Minutes (0-59): 0
---------------------------------------------------------------
Confirm your entry!
  Duration: 8 hours and 0 minutes
  Day&Date: Sun, 2024-01-28
  Activity: MainJob
---------------------------------------------------------------
Is above information correct? (y/n): y
---------------------------------------------------------------
Your entry has successfully been added

/* back to main menu */
```

Entering a time for `2) Yesterday` works just the same. Specifying a data
manually let's you enter times for any day whatsoever, be careful to make valid
entries here.

```
/* for which day you want to enter a time menu from screen before */

Your input: 3
Enter year, month, day manually:
  Year (2000 - 2024): 2024
  Month (01 - 12): 01
  Day (01 - 31): 10
Enter duration (hours and minutes):
  Hours   (0-23): 5
  Minutes (0-59): 45
---------------------------------------------------------------
Confirm your entry!
  Duration: 5 hours and 45 minutes
  Day&Date: Wed, 2024-01-10
  Activity: GigHomepageForSusie
---------------------------------------------------------------
Is above information correct? (y/n): y
---------------------------------------------------------------
Your entry has successfully been added

/* back to main menu */
```

The application makes sure you make valid entries at each step, denying wrong
input and also won't let you enter times for future dates (it's a tracker, not
a scheduler).

### 3) delete entry

Ideally this should never be needed. But maybe you kept the timer running,
picked a wrong activity ID for a time that was entered into the database, et
cetera. It's helpful to have the ability to at least delete within the latest
entries. Supported are deletions for up to 7 days prior (8 days in total, today
included).

```
Your option: 3


Entry deletion supported for today and up to 7 days prior
---------------------------------------------------------------
#0	Date: Sun 2024-01-28, hours:  8.00, Activity: MainJob
/* #1 ...
 * #2 ...
 * #3 ...
 * lists all relevant times from database
 * ...
 */
---------------------------------------------------------------

Specify a valid entry number
  'q' to go back to main
Your input: #0
---------------------------------------------------------------
Sure you want to remove the following entry:
#0	Date: Sun 2024-01-28, hours: 8.002222, Activity: MainJob
---------------------------------------------------------------
Should the above entry be removed? (y/n): y
---------------------------------------------------------------
Entry removed
---------------------------------------------------------------
```

### 4) stats

What I want to check most is my hours worked in total, a few averages and
that's it.

```
/* main menu */

Your option: 4


---------------------------------------------------------------
Today:       8.00 (last 5 day avg: 9.73)

Current week
 -> total   15.23
 -> avg/d    7.12
Last week
 -> total   62.65
 -> avg/d    8.89
Last 6 weeks		/* may be <6 if not enough entries in db */
 -> tot/w   56.54
 -> avg/d    8.07
---------------------------------------------------------------
This month total:    132.57
This month avg/day:   12.05
---------------------------------------------------------------
Print detailed statistics per activity? (y/n): /* see next screen */
```

I'm usually not interested in the exact split of my work time, but as stated
before, especially if you add temporary activities for projects, gigs, then
this data can be useful:

```
---------------------------------------------------------------
Print detailed statistics per activity? (y/n): y
---- Activity MainJob
-----------------------------------------------------------
Today:       8.00 (last 5 day avg: 8.5)
All time:  823.75

Current week
 -> total   12.00
 -> avg/d    6.00
Last week
 -> total   55.00
 -> avg/d    7.85
Last 6 weeks
 -> tot/w   260.00
 -> avg/d    43.33
-----------------------------------------------------------
---- Activity GigHomepageForSusie

/* and for ALL currently activated activities
 * ...
 * ...
 * ... and back to main menu in the end
 */
```

### 5) stats (yearly)

Those statistics are a bit of interesting fun if you've used the program
diligently for a while.

If you only have data for the current year, it'll only show you you statistics
on the current year:

```
/* main menu */

Your option: 5

This year
  % of days  w/  0 hrs :   7.14
  % of days  w/  4 hrs+:  89.28
  % of days  w/  8 hrs+:  71.42
  % of days  w/ 10 hrs+:  46.42
  % of days  w/ 12 hrs+:  17.85
  % of weeks w/  0 hrs :   5.00
  % of weeks w/ 20 hrs+:  95.00
  % of weeks w/ 40 hrs+:  92.50
  % of weeks w/ 50 hrs+:  75.00
  % of weeks w/ 60 hrs+:  37.50
  % of weeks w/ 70 hrs+:  22.50
  % of weeks w/ 80 hrs+:  17.50
---------------------------------------------------------------

/* back to main menu */
```

If, however, you have entries for the last year (and potentially subsequent
years), it will show you `Last year` and `ALL TIME` statistics as well.

```
/* main menu */

Your option: 5

This year
  % of days  w/  0 hrs :   7.14
  % of days  w/  4 hrs+:  89.28
  % of days  w/  8 hrs+:  71.42
  % of days  w/ 10 hrs+:  46.42
  % of days  w/ 12 hrs+:  17.85
  % of weeks w/  0 hrs :   5.00
  % of weeks w/ 20 hrs+:  95.00
  % of weeks w/ 40 hrs+:  92.50
  % of weeks w/ 50 hrs+:  75.00
  % of weeks w/ 60 hrs+:  37.50
  % of weeks w/ 70 hrs+:  22.50
  % of weeks w/ 80 hrs+:  17.50
Last year
  /* ommitted
   * same data points as for `This year`
   * ...
   */
ALL TIME:
  /* ommitted
   * same data points as for `This year`
   * ...
   */

/* back to main menu */
```

### 6) configuration of activities

```
/* main menu */

Your option: 6

Options:

  (a)dd new
  (d)eactivate
  (r)eactivate
  (q)uit (back to main menu)

Your option:
```

#### `(a)dd new` we've seen before, but to showcase again:

```
Options:

  (a)dd new
  (d)eactivate
  (r)eactivate
  (q)uit (back to main menu)

Your option: a
Enter activity name: New super cool project
Options:

/* menu again */
```

#### `(d)eactivate`

Deactivating an activity has the following consequences:

* frees up the associated ID integer (in fact reshuffles your IDs for active
  activities based on the freed up activity ID)
* keeps the activity out of your way in all relevant menues (including not
  listing them in the `stats per activity` output)

But (of course):

* associated times tracked are still part of all your daily/weekly/yearly
  statistics

```
/* configuration menu */

Your option: d
---------------------------------------------------------------
ID	Name
1	Main Job
2	Weekend Project
3	Susie's Homepage
---------------------------------------------------------------
Enter one of the listed activity IDs
  'q' to go back to main

Your input: 3
Activity deactivated

/* configuration menu */
```

#### `(r)eactivate`

```
/* configuration menu */

Your option: r
---------------------------------------------------------------
ID	Name
-1	Pete's Project
-2	Susie's Homepage
-3	rust cli tracker
---------------------------------------------------------------
Enter one of the listed activity IDs
  'q' to go back to main

Your input: -2
Activity reactivated

/* configuration menu */
```

This means `Susie's Homepage` shows up in all menues, is reassigned to a
positive id number again, et cetera... for example in the tracking menu now:

```
/* main menu */

Your option: 1

---------------------------------------------------------------
ID	Name
1   Main Job
2   Weekend Project
3   Susie's Homepage
---------------------------------------------------------------
Enter one of the listed activity IDs
  'q' to go back to main

Your input: 3
Press Enter to switch between work/break
Press q-Enter to end
Started work timer
q 00:00:01
Work time thus far: 00:00:01

Total worked:	00:00:01
Total paused:	00:00:00
Pause percentage: 0.00%

/* main menu */
```

### 7) exit

Simply exists the application:

```
Your option: 7

usr@machine ~/g/w/timetracker/debug (master)> /* back in my console */
```

Note: There's no special shutdown routine or anything, so from the main menu
you can just as easily quit with `Ctrl-C` on Linux and it won't have any
negative consequences. It's perfectly fine (I do it myself):

```
Your option: ^C⏎                                                                                
usr@machine ~/g/w/timetracker (master) [SIGINT]>
```

## Download, build, run

Grab the code from [github.com/srcsinthheta](https://github.com/srcsintheta/timetracker)

Ideally you have Cargo and Rust installed on your system. Simply download the
source code and within the directory:

```
$ ~/git/timetracker (master)> cargo run
```

This builds and runs the project; the binary is under
`target/debug/timetracker`; copy or symlink to it it freely to/from anywhere on
your system.

## Details of time handling

Time's a messy thing... information for the curious...

### track time across midnight; timezone updates

This application handles midnight turnover correctly (if the timer runs past
midnight), attributing the correct time to the previous and to the current day.
It also handles changes to localtime due to DST or a TimeZone change correctly,
simply via checking the offset to UTC for the start and end times.

If you begin tracking a time at UTC+1 and your end time is at UTC-10/UTC+10 (as
in, working on a plane, you end the time when your machine's time has adjusted
to the new local time) w/ potentially even your date having changed, of course
attribution of the measured time to an exact date is compromised in any case,
and a matter of opinion rather than logic. For most users the details of such a
wild scenario shouldn't matter, but it will not wildly corrupt your data or
anything bad as that.

### maximum continuous breakless time

This application DOES NOT handle times measured which go beyond 24 continuous
hours. If the timer runs past 24 hours and is stopped, any such time will
simply not be entered into the database. The technical limitation is handling
two day turnovers, so not being able to run the timer from, say, Monday to
Wednesday. The imposed 24 hour limit is easier to test for, and anything over a
*realistic continuous breakless work phase* (I consider past 24 hours highly
unrealistic) is outside the use case of this application in any case.

Much more likelier is, that you forgot to ever stop the timer if you ever go
past the 24 hour barrier.

## Details on statistics

For your first week's/month's/year's statistics we'll consider the total number
of days from your first use of the application till the end of the
week/month/year (or alternativaly, if this is earlier, till the current day
(excluded)), rather than the total number of days in the week/month/year.

An example to illustrate:

You start using the application on Thursday, the 19th of September 2024, if
week, month, year are finished:

* Weekly  average for first week:  `total / 4`   instead of `/ 7`
* Monthly average for first month: `total / 12`  instead of `/ 30`
* Yearly  average for first year:  `total / 104` instead of `/ 366` (leap year)

Considered this the best way to handle this. Otherwise, to illustrate, if you
work 9 hours on average but have started to use the application on 19th of
September, your year's average would be `936 / 366` and show up as `2.557377`.

Another example:

You started using the application on Tuesday, 2024-01-23, and are prompting the
application for stats on Sunday just a few days later.

* Total today is of course numbers accumulated today (Sunday)
* Total for this week are Tue to Sat; average is `total / 5` (Mon&Sun excluded)
* Total for this month is Tue to Sat; average is `total / 5` (again)

Since you started using the application on Tuesday, the Monday without entries
shouldn't push your averages down; also excluding today makes sense to me since
the day isn't finished until it's over.

#### daily statistics

For daily statistics specifically, your current day is always excluded (since
it hasn't finished yet). Of course once the day is past, it's part of the
statistics as expected.

#### weekly statistics

For weekly statistics (and only affecting the weekly stats, not the day
stats) partial weeks are not considered

* if your very first entry is a Tuesday, the week is disregarded rather than
  counted (of course the days count towards the day stats); if your very first
  entry is a Monday I decided to take the week into account
* your current week is also disregarded, since it's not considered to be
  finished (of course, again, days count (except your current day, see previous
  section), and week becomes part of week stats once week has finished)

## Details on reasoning behind de-/reactivation

The reason you can not delete activities, and can merely deactivate them, is
that a tracker, above all, should keep data consistently and in a relatable way.

If I keep the times tracked themselves but delete the associated activities, I
have data that can't be associated w/ an activity anymore. I also have to
permanently make the low ID number associated w/ the deleted activity unusable,
or need another scheme to change their entries in the database. If I also
delete the times tracked, then the database will constantly be purged from
significant portions of its data, and not provide useful long-term statistics.

Deactivation gets activities out of your way when you're done w/ them. But the
db itself never loses track what tracked time belongs to which activity, nor
does it ever delete entries here.

## Future plans

* better commenting of the source code
* small tweaks to the UI here or there
* cleaning up the codebase in general
* bugfixes when found/reported (some bugs you always find first in extended use)
* improvement in modularization (using templates)

## Potential future

* refining error handling?
* transition to using a TUI?
* transition to using a GUI? (unlikely, could be fun though)

Adding more features per se is not planned as of writing as I don't need them &
am quite happy with the current capabilities. I could've added so much more
functionality, of course, but in the end it's a tool for me to use, and I've
got more than enough interesting things to do.

When my needs change, there's no telling where I could take this... if I
diverge greatly from what it is now I might just create a new project though.

## Feedback

It's welcome, for ways to reach me see [sintheta.dev](https://sintheta.dev)

