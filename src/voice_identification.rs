//! Pirro voice identification. Immutable candidate sets; all 2^31 states, no sampling.
//! A candidate is the first special-check state q = T^3(start), a bijection mod 2^31.
//! After S the next check is T^3(q); after N it is T^4(q).
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

const MASK: u32 = 0x7fff_ffff;
const WORDS: usize = (1usize << 30) / 64;
const SPARSE_LIMIT: u64 = 1 << 24;
const CANCELLED: i32 = 1;

fn next(s: u32) -> u32 { s.wrapping_mul(214013).wrapping_add(2531011) }
fn jump(mut s: u32, mut n: u64) -> u32 {
    let (mut a, mut b) = (214013u32, 2531011u32);
    while n != 0 {
        if n & 1 != 0 { s = a.wrapping_mul(s).wrapping_add(b); }
        b = a.wrapping_add(1).wrapping_mul(b); a = a.wrapping_mul(a); n >>= 1;
    }
    s
}
fn position(mut target: u32) -> u32 {
    target &= MASK;
    let (mut s, mut a, mut b, mut p) = (0u32,214013u32,2531011u32,0u32);
    for bit in 0..31 {
        if (s ^ target) & (1 << bit) != 0 {
            s = a.wrapping_mul(s).wrapping_add(b); p |= 1 << bit;
        }
        b = a.wrapping_add(1).wrapping_mul(b); a = a.wrapping_mul(a);
    }
    debug_assert_eq!(s & MASK,target);
    p
}
#[derive(Clone,Copy)]
struct Affine { a: u32, b: u32 }
impl Affine {
    fn step(n: u64) -> Self { let b=jump(0,n); Self{a:jump(1,n).wrapping_sub(b),b} }
    fn apply(self,q:u32)->u32 { self.a.wrapping_mul(q).wrapping_add(self.b) }
    fn then(self,t:Self)->Self { Self{a:t.a.wrapping_mul(self.a),b:t.a.wrapping_mul(self.b).wrapping_add(t.b)} }
    fn inverse(self,q:u32)->u32 {
        let mut inv=self.a;
        for _ in 0..5 { inv=inv.wrapping_mul(2u32.wrapping_sub(self.a.wrapping_mul(inv))); }
        q.wrapping_sub(self.b).wrapping_mul(inv)&MASK
    }
}
fn normal(q:u32)->bool { q & 0x4000_0000 != 0 }

enum Candidates {
    All,
    Half(u32), // Selected first-observation bit; low 30 bits remain implicit.
    Dense { root:u32, bits:Box<[u64]>, count:u64 },
    Sparse(Box<[u32]>),
}
impl Candidates {
    fn count(&self)->u64 { match self { Self::All=>1<<31,Self::Half(_)=>1<<30,Self::Dense{count,..}=>*count,Self::Sparse(v)=>v.len() as u64 } }
    fn first(&self,limit:usize)->Vec<u32> {
        match self {
            Self::All=>(0..limit.min(1usize<<31)).map(|x|x as u32).collect(),
            Self::Half(root)=>(0..limit.min(1usize<<30)).map(|x|root|x as u32).collect(),
            Self::Sparse(v)=>v.iter().take(limit).copied().collect(),
            Self::Dense{root,bits,..}=>{
                let mut out=Vec::with_capacity(limit);
                for (i,&word) in bits.iter().enumerate() {
                    let mut w=word;
                    while w!=0 {
                        if out.len()==limit { return out; }
                        let bit=w.trailing_zeros();out.push(root|(i as u32*64+bit));w&=w-1;
                    }
                }
                out
            }
        }
    }
}
pub struct Snapshot { candidates:Candidates, at:Affine, observations:u32, draws:u64 }
impl Snapshot {
    fn start()->Self { Self{candidates:Candidates::All,at:Affine{a:1,b:0},observations:0,draws:0} }
}
pub struct Control { cancelled:AtomicBool, done:AtomicU64, total:AtomicU64 }
impl Control {
    fn new()->Self { Self{cancelled:AtomicBool::new(false),done:AtomicU64::new(0),total:AtomicU64::new(1)} }
    fn check(&self)->Result<(),i32> { if self.cancelled.load(Ordering::Relaxed) {Err(CANCELLED)}else{Ok(())} }
}
fn worker_count()->usize { std::thread::available_parallelism().map_or(1,|x|x.get()).min(8) }

fn filter(source:&Snapshot,want_normal:bool,control:&Control)->Result<Snapshot,i32> {
    control.check()?;
    let at=source.at;
    let candidates=match &source.candidates {
        Candidates::All=>Candidates::Half(if want_normal {0x4000_0000}else{0}),
        Candidates::Sparse(input)=>{
            control.total.store(input.len().max(1) as u64,Ordering::Relaxed);
            let chunk_size=input.len().max(1).div_ceil(worker_count());
            let parts=std::thread::scope(|scope| {
                let workers:Vec<_>=input.chunks(chunk_size).map(|part|scope.spawn(move||{
                    let mut out=Vec::with_capacity(part.len()/2+1);
                    for block in part.chunks(4096) {
                        control.check()?;
                        out.extend(block.iter().copied().filter(|&q|normal(at.apply(q))==want_normal));
                        control.done.fetch_add(block.len() as u64,Ordering::Relaxed);
                    }
                    Ok(out)
                })).collect();
                workers.into_iter().map(|w|w.join().expect("filter worker failed")).collect::<Result<Vec<_>,i32>>()
            })?;
            let mut out=Vec::with_capacity(parts.iter().map(Vec::len).sum());
            for p in parts { out.extend(p); }
            Candidates::Sparse(out.into_boxed_slice())
        },
        Candidates::Half(root)|Candidates::Dense{root,..}=>{
            control.total.store(WORDS as u64,Ordering::Relaxed);
            let input=if let Candidates::Dense{bits,..}=&source.candidates {Some(bits.as_ref())} else {None};
            let mut output=vec![0u64;WORDS];
            let chunk_size=WORDS.div_ceil(worker_count());
            let counts=std::thread::scope(|scope| {
                let workers:Vec<_>=output.chunks_mut(chunk_size).enumerate().map(|(worker,part)|scope.spawn(move||{
                    let mut count=0u64;
                    for (block_index,block) in part.chunks_mut(4096).enumerate() {
                        control.check()?;
                        let start=worker*chunk_size+block_index*4096;
                        for (offset,out) in block.iter_mut().enumerate() {
                            let i=start+offset;
                            let mut word=input.map_or(u64::MAX,|v|v[i]);
                            let mut found=0u64;
                            while word!=0 {
                                let bit=word.trailing_zeros();
                                let q=root|(i as u32*64+bit);
                                if normal(at.apply(q))==want_normal {found|=1u64<<bit;}
                                word&=word-1;
                            }
                            *out=found;count+=u64::from(found.count_ones());
                        }
                        control.done.fetch_add(block.len() as u64,Ordering::Relaxed);
                    }
                    Ok(count)
                })).collect();
                workers.into_iter().map(|w|w.join().expect("filter worker failed")).collect::<Result<Vec<_>,i32>>()
            })?;
            let count=counts.iter().sum();
            if count<=SPARSE_LIMIT {
                let mut sparse=Vec::with_capacity(count as usize);
                for (i,&word) in output.iter().enumerate() {
                    if i%65536==0 {control.check()?;}
                    let mut w=word;
                    while w!=0 {sparse.push(root|(i as u32*64+w.trailing_zeros()));w&=w-1;}
                }
                assert_eq!(sparse.len() as u64,count);
                Candidates::Sparse(sparse.into_boxed_slice())
            } else {Candidates::Dense{root:*root,bits:output.into_boxed_slice(),count}}
        }
    };
    control.check()?;
    let calls=if want_normal {4}else{3};
    Ok(Snapshot{candidates,at:at.then(Affine::step(calls)),observations:source.observations+1,draws:source.draws+calls})
}

#[derive(serde::Serialize, Clone, Copy)]
pub struct Candidate { initial_state:u32,current_state:u32,initial_position:u32,current_position:u32,next_rand:u32,next_normal:u32 }
fn row(s:&Snapshot,q:u32)->Candidate {
    let initial_state=Affine::step(3).inverse(q);
    let current_state=jump(initial_state,s.draws)&MASK;
    let initial_position=position(initial_state);
    Candidate{initial_state,current_state,initial_position,current_position:initial_position.wrapping_add(s.draws as u32)&MASK,
        next_rand:(next(current_state)>>16)&0x7fff,next_normal:u32::from(normal(s.at.apply(q)))}
}

/// Cache only the latest exact transcript; edits/undo rebuild from all states.
pub fn identify(voices: &str) -> crate::Result<serde_json::Value> {
    use std::sync::{Mutex, OnceLock};
    crate::ensure(voices.len() <= 512 && voices.bytes().all(|b| matches!(b,b'N'|b'S')),
        "Voice transcript must contain at most 512 N/S observations")?;
    static CACHE: OnceLock<Mutex<(String,Snapshot)>> = OnceLock::new();
    let mut cache=CACHE.get_or_init(||Mutex::new((String::new(),Snapshot::start())))
        .lock().map_err(|_|crate::SimError("Voice identifier unavailable; restart the server".into()))?;
    if !voices.starts_with(&cache.0) { *cache=(String::new(),Snapshot::start()); }
    for b in voices[cache.0.len()..].bytes() {
        let next=filter(&cache.1,b==b'N',&Control::new())
            .map_err(|_|crate::SimError("Voice filtering cancelled".into()))?;
        cache.1=next;cache.0.push(b as char);
    }
    let s=&cache.1;
    let candidate=(s.candidates.count()==1).then(||row(s,s.candidates.first(1)[0]));
    Ok(serde_json::json!({"count":s.candidates.count(),"observations":s.observations,
        "draws":s.draws,"candidate":candidate,"voices":voices}))
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn transcript_tracks_current_not_initial_seed() {
        for seed in [0,1,1234567,0x7fff_ffff] {
            let mut s=Snapshot{candidates:Candidates::Sparse(vec![jump(seed,3)&MASK].into_boxed_slice()),
                at:Affine{a:1,b:0},observations:0,draws:0};
            let mut expected=seed;
            for _ in 0..48 {
                let n=normal(jump(expected,3));
                s=filter(&s,n,&Control::new()).unwrap();expected=jump(expected,if n {4}else{3})&MASK;
                let r=row(&s,s.candidates.first(1)[0]);
                assert_eq!(r.initial_state,seed);assert_eq!(r.current_state,expected);
                assert_eq!(jump(0,r.current_position.into())&MASK,expected);
            }
        }
    }
    #[test] fn rejects_invalid_transcripts(){assert!(identify("NSX").is_err());}
}
