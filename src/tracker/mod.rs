//! main tracker run loop and update sql table functionality

use std::error;
use std::io::{self, Write};
use std::sync::{Arc, atomic::AtomicBool, atomic::Ordering};
use std::thread;
use std::time::Duration;

/* A previous implementation returned the seconds (via a since removed custom
 * struct denoting a time duration, just a side note) from the workloop to be
 * used to denote time passed;
 * I don't expect thread::sleep to be an accurate measurement though
 *
 * we now retrieve local time via chrono beginning and end
 * (not here, in lib.rs / db module) to calculate time passed
 */

pub fn workloop() 
    -> Result<bool, Box<dyn error::Error>>
{
    let shouldrun = Arc::new(AtomicBool::new(true));
    let mut done = false;

    // variables to be captured by timer_thread closure
    let shouldrun_clone = Arc::clone(&shouldrun);

    // timer clock thread
    let timer_thread = thread::spawn(move || {

        let mut seconds : u32 = 0;
        let mut hh : u32;
        let mut mm : u32;
        let mut ss : u32;
        let mut count = 0;

        while shouldrun_clone.load(Ordering::SeqCst)
        {
            thread::sleep(Duration::from_millis(100));
            count += 1;

            if count == 10
            {
                seconds += 1;
                hh = seconds / 3600;
                mm = seconds % 3600 / 60;
                ss = seconds % 60;

                print!("  {:02}:{:02}:{:02}\r", hh, mm, ss);
                io::stdout().flush().unwrap();
                count = 0;
            }
        }
    });

    let mut input = String::new();
    while input != "\n" && input.trim() != "q"
    {
        input.clear();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).expect("Failed to read line");
    }

    if input.trim() == "q"
    {
        done = true;
    }

    shouldrun.store(false, Ordering::SeqCst); 		 // AtomicBool to false

    if let Err(e) = timer_thread.join()
    {
        println!("Timer thread error: {:?}", e);
    }

    Ok(done)
}
