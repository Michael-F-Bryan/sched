//! Functions and types that are related to creating a representation of
//! a task which is to be run at some time in the future.


use std::fmt::{Formatter, Debug, Error};
use chrono::{Duration, Local, DateTime};


/// An alias for a boxed closure.
pub type Func = Box<Fn() + Send + Sync>;

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum TimeSpan {
    Millisecond,
    Milliseconds,
    Second,
    Seconds,
    Minute,
    Minutes,
    Hour,
    Hours,
    Day,
    Days,
    Week,
    Weeks,
}


/// A task that is designed to be run at some point in the future.
pub struct Job {
    duration: Duration,
    last_run: DateTime<Local>,
    next_run: Option<DateTime<Local>>,
    once_off: bool,
    name: Option<String>,
    func: Option<Func>,
    times_run: u32,
}

impl Debug for Job {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self.name {
            Some(ref n) => write!(f, "Job(name='{}')", n),
            None => write!(f, "Job(name='UNKNOWN')"),
        }
    }
}

impl Default for Job {
    fn default() -> Job {
        Job {
            last_run: Local::now(),
            next_run: None,
            duration: Duration::zero(),
            once_off: false,
            name: None,
            func: None,
            times_run: 0,
        }
    }
}

impl Job {
    /// Construct a bare Job.
    pub fn new() -> Self {
        Job::default()
    }

    /// Give the job a name.
    pub fn name(mut self, s: &str) -> Job {
        self.name = Some(s.to_string());
        self
    }

    /// Construct a periodic job.
    pub fn every(n: i64, delta_type: TimeSpan) -> Job {
        let mut d = Job::new();
        d.increment(n, delta_type);
        d
    }

    /// Create a once off job.
    pub fn in_(n: i64, delta_type: TimeSpan) -> Job {
        let mut d = Job::new();
        d.increment(n, delta_type);
        d.once_off = true;
        d
    }

    /// Add to the duration between runs.
    ///
    ///     use sched::*;
    ///     let job = Job::every(5, Minutes).and(10, Seconds);
    pub fn and(mut self, n: i64, delta_type: TimeSpan) -> Job {
        self.increment(n, delta_type);
        self
    }

    /// Increase the duration between runs by a certain amount.
    fn increment(&mut self, n: i64, delta_type: TimeSpan) {
        let new_duration = match delta_type {
            TimeSpan::Millisecond | TimeSpan::Milliseconds => Duration::milliseconds(n),
            TimeSpan::Second | TimeSpan::Seconds => Duration::seconds(n),
            TimeSpan::Minute | TimeSpan::Minutes => Duration::minutes(n),
            TimeSpan::Hour | TimeSpan::Hours => Duration::hours(n),
            TimeSpan::Day | TimeSpan::Days => Duration::days(n),
            TimeSpan::Week | TimeSpan::Weeks => Duration::weeks(n),
        };


        self.duration = self.duration + new_duration;

        // Update the next_run
        self.next_run = Some(self.last_run + self.duration);
    }

    /// Give the job a closure to run and validate that everything has been
    /// entered correctly.
    pub fn do_(mut self, f: Func) -> Result<Job, String> {
        self.func = Some(f);
        self.validate()
    }

    /// Check that a job is valid and ready to be run.
    fn validate(self) -> Result<Self, String> {
        if self.duration.is_zero() {
            Err("No duration entered".to_string())
        } else if self.func.is_none() {
            Err("No function supplied".to_string())
        } else {
            Ok(self)
        }
    }

    /// Check if a job is periodic or once off.
    pub fn is_periodic(&self) -> bool {
        !self.once_off
    }

    /// Get the number of times the job has been run.
    pub fn times_run(&self) -> u32 {
        self.times_run
    }

    /// Run the job and update the metadata recording when the last time this
    /// job was run.
    pub fn execute(&mut self) -> Result<(), String> {
        self.last_run = Local::now();

        // Update the next run or set it to None if this was a
        // once off job
        if self.once_off {
            self.next_run = None;
        } else {
            self.next_run = Some(Local::now() + self.duration);
        }

        match self.name {
            Some(ref name) => info!("Running {}", name),
            None => (),
        }

        match self.func {
            Some(ref f) => {
                f.call(());
                self.times_run += 1;
                Ok(())
            }

            None => Err("No function provided!".to_string()),
        }
    }

    /// Check whether the job needs to be run.
    pub fn ready(&self) -> bool {
        match self.next_run {
            Some(next) => next <= Local::now(),
            None => false,
        }
    }

    /// The next time this job should be run.
    pub fn next_run(&self) -> Option<DateTime<Local>> {
        self.next_run
    }
}



#[cfg(test)]
mod tests {
    use super::Job;
    use std::sync::Mutex;
    use std::sync::Arc;
    use super::TimeSpan::*;
    use chrono::{Duration, Local};

    #[test]
    fn constructor() {
        let got = Job::new();
        assert!(got.duration.is_zero());
        assert!(got.name.is_none());
        assert!(!got.once_off);
    }

    #[test]
    fn ideal_use() {
        let job = Job::every(5, Minutes).do_(Box::new(|| println!("Hello World!"))).unwrap();
        assert!(job.is_periodic());

        let duration = Duration::minutes(5);
        assert_eq!(job.duration, duration);

        assert!(job.func.is_some());
    }

    #[test]
    fn check_if_periodic() {
        let mut job = Job::new();
        assert!(!job.once_off);
        assert!(job.is_periodic());

        job.once_off = true;
        assert!(job.once_off);
        assert!(!job.is_periodic());
    }

    #[test]
    fn validate_invalid_job() {
        let job = Job::new();
        assert_eq!(job.validate().unwrap_err(),
                   "No duration entered".to_string());

        let job = Job::every(5, Minutes);
        assert_eq!(job.validate().unwrap_err(),
                   "No function supplied".to_string());
    }

    #[test]
    fn run_the_job() {
        // Create a reference counted number and pass it to the closure
        let num: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let num_2 = num.clone();

        // Create a job that runs every 5 minutes and will simply increment
        // our number (which is wrapped in an Arc and a mutex
        let mut job = Job::every(5, Minutes)
            .do_(Box::new(move || {
                let mut n = num_2.lock().unwrap();
                *n = 42;
            }))
            .unwrap();

        // Check the job has never been run
        assert_eq!(job.times_run(), 0);

        // Now actually run the job;
        job.execute().unwrap();

        assert_eq!(*num.lock().unwrap(), 42);

        // Make sure the run counter increased
        assert_eq!(job.times_run(), 1);
    }

    #[test]
    fn increment_with_and() {
        let job = Job::every(5, Minutes).and(18, Seconds);
        let duration = Duration::minutes(5) + Duration::seconds(18);
        assert_eq!(job.duration, duration);
    }

    #[test]
    fn check_if_ready() {
        let mut job = Job::every(5, Minutes).and(18, Seconds);
        assert!(job.is_periodic());
        assert!(job.next_run.unwrap() > Local::now());
        assert!(!job.ready());

        // Now change the job's next run time to some time in the "past"
        job.next_run = Some(Local::now());
        assert!(job.ready());
    }

    #[test]
    fn make_sure_once_off_only_executes_once() {
        let mut job = Job::in_(1, Second)
            .do_(Box::new(|| {
                1 + 1;
            }))
            .unwrap();
        job.execute().unwrap();
        assert!(!job.ready());
    }
}
