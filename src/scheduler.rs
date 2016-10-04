//! A scheduler who is in charge of checking whether a job is ready to be run
//! and then executing it in the background on another thread.

use std::fmt::{Formatter, Debug, Error};
use chrono::{Duration, Local};
use std::thread;
use super::Job;


/// A job scheduler
#[derive(Default)]
pub struct Scheduler {
    job_queue: Vec<Job>,
}

impl Debug for Scheduler {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Scheduler(job_queue={:?})", self.job_queue)
    }
}

impl Scheduler {
    /// Create a new scheduler with an empty job queue.
    pub fn new() -> Scheduler {
        Scheduler::default()
    }

    /// Add a Job to the job queue.
    pub fn add_job(&mut self, job: Job) {
        self.job_queue.push(job);
    }

    /// Check if there are any jobs that need to be run.
    pub fn pending(&self) -> bool {
        self.job_queue.iter().any(|j| j.ready())
    }

    /// Get the time until the next job is due to be executed.
    pub fn time_to_next(&self) -> Option<Duration> {
        let times: Vec<_> = self.job_queue.iter().filter(|j| j.next_run().is_some())
            .map(|j| Local::now() - j.next_run().unwrap())
            .collect();

        times.iter().max().map(|t| t.clone())
    }

    /// Run any pending jobs and return the number of jobs run.
    pub fn run_pending(&mut self) -> usize {
        let mut count = 0;
        for job in &mut self.job_queue {
            if job.ready() {
                count += 1;
                let result = job.execute();

                if result.is_err() {
                    error!("{}", result.unwrap_err());
                }
            }
        }

        count
    }

    /// Run the jobs forever.
    pub fn run_forever(&mut self) {
        loop {
            let delay = self.time_to_next();
            match delay {
                Some(wait_duration) => {
                    thread::sleep(wait_duration.to_std().unwrap());
                    self.run_pending();
                },
                None => break
            }
        }
    }
}


#[cfg(test)]
mod test {
    use std::thread::sleep;
    use std::time::Duration as Duration_std;
    use std::sync::{Mutex, Arc};
    use super::Scheduler;
    use super::super::Job;
    use super::super::TimeSpan::*;

    #[test]
    fn constructor() {
        let sched = Scheduler::new();
        assert!(sched.job_queue.is_empty());
    }

    #[test]
    fn add_job_to_queue() {
        let job = Job::every(5, Minutes).run(Box::new(|| ())).unwrap();
        let job_2 = Job::every(5, Minutes).run(Box::new(|| ())).unwrap();
        let mut sched = Scheduler::new();
        assert!(sched.job_queue.is_empty());

        sched.add_job(job);
        assert_eq!(sched.job_queue.len(), 1);

        // Add a second job
        sched.add_job(job_2);
        assert_eq!(sched.job_queue.len(), 2);
    }

    #[test]
    fn is_empty_queue_pending() {
        let sched = Scheduler::new();
        assert!(sched.job_queue.is_empty());
        assert!(!sched.pending());
    }

    #[test]
    fn queue_with_pending_task() {
        let job = Job::every(10, Milliseconds).run(Box::new(|| ())).unwrap();

        // Wait until after the job is ready
        sleep(Duration_std::from_millis(11));
        assert!(job.ready());

        let mut sched = Scheduler::new();
        assert!(sched.job_queue.is_empty());
        assert!(!sched.pending());

        sched.add_job(job);
        assert!(sched.pending());
    }

    #[test]
    fn execute_jobs() {
        // Create a reference counted number and pass it to the closure
        let num: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let num_2 = num.clone();

        let job = Job::every(10, Milliseconds)
            .run(Box::new(move || {
                let mut n = num_2.lock().unwrap();
                *n = 42;
            }))
            .unwrap();

        // Make sure the number hasn't been changed
        {
            assert_eq!(0, *num.lock().unwrap());
        }

        // Wait until after the job is ready
        sleep(Duration_std::from_millis(11));
        assert!(job.ready());

        let mut sched = Scheduler::new();
        sched.add_job(job);

        // Run the pending jobs
        let num_run = sched.run_pending();
        assert_eq!(num_run, 1);

        // Make sure the job actually changed our number
        assert_eq!(*num.lock().unwrap(), 42);
    }
}
