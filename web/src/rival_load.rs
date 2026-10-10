//! The AI's rival robot (user, 2026-10-10): a public robot the page picks
//! from its dropdown, fetched in the background and kept, so switching
//! back to it rebuilds at once.

use crate::bundle_load::{Bundle, fetch_bundle};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::wasm_bindgen;

/// The page's latest pick: the rival's bundle base URL, "" for the
/// robot's own copy. A static for the same reason as the opponent kind.
static WANT: Mutex<String> = Mutex::new(String::new());

/// Called by index.html's dropdown with a bundle base URL ending in '/'
/// (`/api/robots/<id>/bundle/`), or "" for the robot's own copy.
#[wasm_bindgen]
pub fn set_rival(base: String) {
    *WANT.lock().unwrap() = base;
}

/// The rival the page last picked ("" until it picks one).
pub fn wanted_rival() -> String {
    WANT.lock().unwrap().clone()
}

/// A fetched bundle, or why it could not be loaded.
type Fetched = Result<Bundle, String>;

/// Every rival bundle fetched this session, by base URL.
#[derive(Resource)]
pub struct Rivals {
    motors_url: String,
    fetched: HashMap<String, Fetched>,
    pending: HashSet<String>,
    /// Where the async fetches leave their results.
    done: Arc<Mutex<Vec<(String, Fetched)>>>,
}

impl Rivals {
    /// Rivals load with the motor library the driven robot loaded with.
    pub fn new(motors_url: String) -> Self {
        Rivals {
            motors_url,
            fetched: HashMap::new(),
            pending: HashSet::new(),
            done: Arc::default(),
        }
    }

    /// The bundle at `base` (or why it failed to load), starting its
    /// fetch the first time it is asked for; `None` while it loads.
    pub fn get(&mut self, base: &str) -> Option<&Fetched> {
        for (b, r) in self.done.lock().unwrap().drain(..) {
            self.pending.remove(&b);
            self.fetched.insert(b, r);
        }
        if !self.fetched.contains_key(base) && self.pending.insert(base.to_string()) {
            let (base, motors, done) =
                (base.to_string(), self.motors_url.clone(), self.done.clone());
            wasm_bindgen_futures::spawn_local(async move {
                let r = fetch_bundle(base.clone(), motors).await;
                done.lock().unwrap().push((base, r));
            });
        }
        self.fetched.get(base)
    }
}
