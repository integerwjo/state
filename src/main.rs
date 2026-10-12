use std::{thread, time::Duration};

mod future;
mod http;

use future::{Future, PollState};
use http::Http;

struct Coroutine {
    state: State,
}

impl Coroutine {
    fn new() -> Self {
        Self {
            state: State::Start,
        }
    }
}

impl Future for Coroutine {
    type Output = ();

    fn poll(&mut self) -> PollState<Self::Output> {
        loop {
            match self.state {
                State::Start => {
                    println!("Program starting");
                    let fut1 = Box::new(Http::get("/600/HelloWorld1"));
                    self.state = State::Wait1(fut1);
                }

                State::Wait1(ref mut fut) => match fut.poll() {
                    PollState::Ready(txt) => {
                        println!("{txt}");
                        let fut2 = Box::new(Http::get("/400/HelloWorld2"));
                        self.state = State::Wait2(fut2);
                    }
                    PollState::NotReady => break PollState::NotReady,
                },

                State::Wait2(ref mut fut2) => match fut2.poll() {
                    PollState::Ready(txt) => {
                        println!("{txt}");
                        self.state = State::Resolved;
                        break PollState::Ready(());
                    }
                    PollState::NotReady => break PollState::NotReady,
                },

                State::Resolved => panic!("Polled a resolved future"),
            }
        }
    }
}
enum State {
    // The coroutine has been created but it hasnt been polled yet
    Start,

    // Store the HttpGetFuture returned in the state enum
    Wait1(Box<dyn Future<Output = String>>),
    Wait2(Box<dyn Future<Output = String>>),

    // The future has been resolved and theres no more work to do
    Resolved,
}

fn main() {
    let mut future = async_main();

    loop {
        match future.poll() {
            PollState::NotReady => println!("Schedule other tasks"),
            PollState::Ready(_) => break,
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn async_main() -> impl Future<Output = ()> {
    Coroutine::new()
}

/*
async fn async_main() {
    println!("Program starting");
    let txt = Http::get("/1000/HelloWorld").await;
    println!("{txt}");

    // nina haja nawe
    let txt2 = Http::get("/500/HelloWorld2").await;
    println!("{txt2}");
} */
