extern crate sched;

use sched::*;

fn main() {
    let job = Job::every(5, Seconds).do_(Box::new(|| println!("Hello World"))).unwrap();
    let mut sched = Scheduler::new();
    sched.add_job(job);

    sched.run_forever();
}
