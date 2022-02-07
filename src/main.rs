use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};
use nix::{
    errno::Errno,
    sys::event::{
        KEvent, kqueue, kevent, EventFilter, FilterFlag, EventFlag,
    },
    unistd::write,
};
use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    io::{BufRead, BufReader, BufWriter, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    os::unix::io::{AsRawFd, RawFd},
    pin::Pin,
    slice::from_raw_parts,
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex,
    },
    task::{Context, Poll, Waker},
};

fn write_event_fd(fd: RawFd, n: usize) {
    let ptr = &n as *const usize as *const u8;
    let val = unsafe { from_raw_parts(ptr, std::mem::size_of_val(&n)) };
    write(fd, &val).unwrap();
}

enum IOOps {
    ADD(EventFlag, RawFd, Waker),
    REMOVE(RawFd),
}

struct IOSelector {
    wakers: Mutex<HashMap<RawFd, Waker>>,
    queue: Mutex<VecDeque<IOOps>>,
    kqfd: RawFd,
    event: RawFd,
}

impl IOSelector {
    fn new() -> Arc<Self> {
        let s = IOSelector {
            wakers: Mutex::new(HashMap::new()),
            queue: Mutex::new(VecDeque::new()),
            kqfd: kqueue().unwrap(),
            event: kqueue().unwrap(),
        };
        let result = Arc::new(s);
        let s = result.clone();
        std::thread::spawn(move || s.select());
        result
    }

    fn add_event(
        &self,
        flag: EventFlag,
        fd: RawFd,
        waker: Waker,
        wakers: &mut HashMap<RawFd, Waker>,
    ) {
        let mut events = vec![];
        let mut ev = KEvent::new(
            fd as usize,
            EventFilter::EVFILT_USER,
            flag | EventFlag::EV_ONESHOT,
            FilterFlag::NOTE_NONE,
            0,
            0,
        );
        let _ = kevent(self.kqfd, events.as_slice(), &mut [], 0).unwrap();
        if let Err(err) = kevent(self.kqfd, &[], events.as_mut_slice(), 0) {
            match err {
                _ => panic!("kevent error: {:?}", err),
            }
        }

        assert!(!wakers.contains_key(&fd));
        wakers.insert(fd, waker);
    }

    fn select(&self) {}
}

fn main() {
    println!("Hello, world!");
}
