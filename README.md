Sched - Cron For Humans
=======================

A scheduler for running periodic or once-off tasks designed for humans making
extensive use of the builder pattern.


Usage
-----

From the ground up this library has been designed with readability and
flexibility in mind. To create a job just do:

    use sched::*;

    let some_job = Job::every(5, Minutes)
            .and(30, Seconds)
            .run(Box::new(|| {
            println!("Hello World");
            }).unwrap();


Installation
------------

Installation is also fairly simple, first grab the code:

    git clone

And install it:

    cargo install

A couple unstable features are used, so you may need to be using the nightly
Rust compiler. If you don't have it yet, just go to [rustup.rs][rustup].


[rustup]: https://www.rustup.rs/
