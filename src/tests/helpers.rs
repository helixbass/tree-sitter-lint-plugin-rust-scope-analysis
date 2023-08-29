#![cfg(test)]

use squalid::run_once;

pub fn tracing_subscribe() {
    run_once! {
        tracing_subscriber::fmt::init();
    }
}
