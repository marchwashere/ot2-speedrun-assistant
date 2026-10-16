//! Cooperative cancellation of disconnected HTTP route searches.
use std::{cell::RefCell,net::TcpStream,time::{Duration,Instant},io::ErrorKind};
use crate::{Result,SimError};
struct Request {socket:TcpStream,last:Instant,cancelled:bool}
thread_local! {static REQUEST:RefCell<Option<Request>>=const {RefCell::new(None)};}
pub struct Guard;
impl Drop for Guard {fn drop(&mut self){REQUEST.with(|r|*r.borrow_mut()=None);}}
pub fn watch(socket:TcpStream)->std::io::Result<Guard>{
    socket.set_read_timeout(Some(Duration::from_millis(1)))?;
    REQUEST.with(|r|*r.borrow_mut()=Some(Request{socket,last:Instant::now()-Duration::from_secs(1),cancelled:false}));
    Ok(Guard)
}
pub fn check()->Result<()> {
    REQUEST.with(|r|{
        let mut r=r.borrow_mut();let Some(r)=r.as_mut() else{return Ok(());};
        if !r.cancelled && r.last.elapsed()>=Duration::from_millis(25) {
            r.last=Instant::now();
            r.cancelled=match r.socket.peek(&mut [0u8;1]) {
                Ok(0)=>true,Ok(_)=>false,
                Err(e)=>!matches!(e.kind(),ErrorKind::WouldBlock|ErrorKind::TimedOut|ErrorKind::Interrupted),
            };
        }
        if r.cancelled {Err(SimError("Route search cancelled: client disconnected".into()))}else{Ok(())}
    })
}
#[cfg(test)] mod tests {
 use super::*;
 #[test]fn disconnect_interrupts_work_and_guard_clears_next_request(){
  let listener=std::net::TcpListener::bind("127.0.0.1:0").unwrap();
  let client=TcpStream::connect(listener.local_addr().unwrap()).unwrap();
  let (server,_)=listener.accept().unwrap();let guard=watch(server).unwrap();
  assert!(check().is_ok());drop(client);
  let start=Instant::now();while check().is_ok(){assert!(start.elapsed()<Duration::from_secs(1));std::thread::sleep(Duration::from_millis(5));}
  drop(guard);assert!(check().is_ok());
 }
}
