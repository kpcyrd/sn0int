use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::result;
use std::sync::mpsc;
use std::time::Duration;

pub type RatelimitSender = mpsc::Sender<result::Result<RatelimitResponse, String>>;

pub struct Ratelimiter {
    buckets: HashMap<String, Bucket>,
}

impl Default for Ratelimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl Ratelimiter {
    pub fn new() -> Ratelimiter {
        Ratelimiter {
            buckets: HashMap::new(),
        }
    }

    pub fn pass(&mut self, tx: RatelimitSender, key: &str, passes: u32, time: u32) {
        let bucket = if let Some(bucket) = self.buckets.get_mut(key) {
            bucket
        } else {
            let bucket = Bucket::new();
            self.buckets.insert(key.to_string(), bucket);
            self.buckets.get_mut(key).unwrap()
        };
        let reply = bucket.pass(passes as usize, time);
        tx.send(Ok(reply)).unwrap();
    }
}

struct Bucket {
    passes: Vec<DateTime<Utc>>,
}

impl Bucket {
    pub fn new() -> Bucket {
        Bucket {
            passes: Vec::new(),
        }
    }

    pub fn pass(&mut self, passes: usize, time: u32) -> RatelimitResponse {
        let now = Utc::now();
        let time = chrono::Duration::milliseconds(time as i64);
        let retain = now - time;
        self.passes.retain(|x| *x >= retain);

        if self.passes.len() >= passes {
            if let Some(min) = self.passes.iter().min() {
                let delay = time - (now - *min);
                RatelimitResponse::Retry(delay.to_std().unwrap())
            } else {
                // This should never happen unless passes is zero
                RatelimitResponse::Retry(Duration::from_millis(100))
            }
        } else {
            let now = Utc::now();
            self.passes.push(now);
            RatelimitResponse::Pass
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RatelimitResponse {
    Retry(Duration),
    Pass,
}
