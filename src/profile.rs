//! Opt-in inclusive search timings (OT2_PROFILE=1), scoped to one request thread.
use std::{cell::RefCell,collections::BTreeMap,sync::OnceLock,time::{Duration,Instant}};
thread_local! {static TIMES:RefCell<BTreeMap<&'static str,(u64,Duration)>>=RefCell::new(BTreeMap::new());}
fn enabled()->bool {static ENABLED:OnceLock<bool>=OnceLock::new();*ENABLED.get_or_init(||std::env::var_os("OT2_PROFILE").is_some())}
pub struct Span {name:&'static str,start:Option<Instant>}
pub fn span(name:&'static str)->Span {Span {name,start:enabled().then(Instant::now)}}
impl Drop for Span {fn drop(&mut self){if let Some(start)=self.start {let elapsed=start.elapsed();TIMES.with(|t|{let mut t=t.borrow_mut();let e=t.entry(self.name).or_default();e.0+=1;e.1+=elapsed;});}}}
pub fn reset(){TIMES.with(|t|t.borrow_mut().clear());}
pub fn report(){TIMES.with(|t|{let t=t.borrow();let mut rows:Vec<_>=t.iter().collect();rows.sort_by_key(|(_,(_,d))|std::cmp::Reverse(*d));for (name,(count,d)) in rows {eprintln!("PROFILE {name}: {count} calls, {:.3}s inclusive",d.as_secs_f64());}});}
