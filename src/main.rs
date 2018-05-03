extern crate actix;
extern crate futures;

use actix::prelude::*;
use futures::{future, Future};

struct Fib(usize);

impl Message for Fib {
    type Result = Result<usize, ()>;
}

struct FibVec(usize);

impl Message for FibVec {
    type Result = Result<Vec<usize>, ()>;
}

struct Fibber();

impl Actor for Fibber {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) { println!("I am fib"); }
}

impl Handler<Fib> for Fibber {
    type Result = ResponseFuture<usize, ()>;

    fn handle(&mut self, msg: Fib, _ctx: &mut Context<Self>) -> Self::Result {
        //std::thread::sleep(std::time::Duration::from_millis(10));
        if msg.0 <= 2 {
            return Box::new(future::ok(msg.0));
        } else {
            let a: Addr<Unsync, _> = (Fibber{}).start();
            let b: Addr<Unsync, _> = (Fibber{}).start();
            let fta = a.send(Fib(msg.0 - 1));
            let ftb = b.send(Fib(msg.0 - 2));

            let ftc = fta.join(ftb).and_then(|(a, b)| {
                let x = a.unwrap();
                let y = b.unwrap();
                future::ok(x + y)
            }).map_err(|_| ());

            return Box::new(ftc);
        }
    }
}

impl Handler<FibVec> for Fibber {
    type Result = ResponseFuture<Vec<usize>, ()>;

    fn handle(&mut self, msg: FibVec, _ctx: &mut Context<Self>) -> Self::Result {
        let worker: Addr<Unsync, _> = (Fibber {}).start();
      
        let numbers = (0..msg.0).map(move |i| 
            worker.send(Fib(i))
            .map_err(|_| ())            // discard any errors, converting E from MailboxErr to ()
            .and_then(future::result)   // convert Result<usize, ()> to Future<usize, ()>
        );

        // collect() converts Vec<Future<T, E>> to Future<Vec<T>, E>
        Box::new(futures::collect(numbers)) 
    }
}

fn main() {
    let system = System::new("Testing");

    let addr: Addr<Unsync, _> = (Fibber {}).start();

    let res = addr.send(FibVec(20));

    system.handle().spawn(res.then(|res| {
        match res {
            Ok(Ok(result)) => println!("FIB: {:?}", result),
            _ => println!("Something went wrong"),
        }

        Arbiter::system().do_send(actix::msgs::SystemExit(0));
        future::ok(())
    }));

    system.run();
}
