//! A sane crate for running jobs periodically. Think Cron, but for humans.


#![feature(fn_traits)]
#![feature(plugin)]

// Define some lints
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(missing_copy_implementations)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]
#![deny(unused_imports)]


extern crate chrono;

pub mod job;

// Re-exports
pub use job::{Job, TimeSpan};
pub use job::TimeSpan::*;
