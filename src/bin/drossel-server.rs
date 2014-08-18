#![crate_type = "bin"]
#![feature(globs,phase)]
//#![phase(syntax, link)] extern crate log;

extern crate green;
extern crate rustuv;
extern crate drossel;
extern crate strand;

use std::io::net::tcp::TcpListener;
use std::io::{Acceptor, Listener};
use std::io::BufferedStream;
use drossel::commands::util::get_command;
use drossel::commands::command::Command;
use drossel::drossel::types::{DBResult,Pong,Inserted,Removed};
use drossel::drossel::db::{DB};
use drossel::drossel::store::*;
use strand::mutable::{Event,AsSendableEvent};
use strand::errors::{Errors};

type ResultType = (Result<DBResult,Errors>);
type ChannelType = (Box<Event<BinaryList, DBResult>+Send>, Sender<ResultType>);

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    green::start(argc, argv, rustuv::event_loop, main)
}

fn main() {
  let (db_sender, db_receiver): (Sender<ChannelType>, Receiver<ChannelType>) = channel();

  // spawn the db task
  spawn(proc() {
    let mut db = DB::new();
    for (event, origin) in db_receiver.iter() {
      origin.send(db.execute(event));
    }
  });

  let listener = TcpListener::bind("127.0.0.1", 7890);

  // bind the listener to the specified address
  let mut acceptor = listener.listen();

  // accept connections and process them
  for stream in acceptor.incoming() {
    let cloned_sender = db_sender.clone();
    spawn(proc() {
      match stream {
        Ok(conn) => {
          let mut buffer = BufferedStream::new(conn);
          let input = buffer.read_until('\n' as u8).unwrap();

          let command: Box<Command> = get_command(input).unwrap();
          let (sender, receiver): (Sender<ResultType>, Receiver<ResultType>) = channel();
          let event = (*command).as_sendable_event();
          cloned_sender.send((event, sender));
          let res = receiver.recv().unwrap();
          let output = match res {
            Pong => { format!("PONG") },
            Inserted(queue) => { format!("OK {}", queue) },
            Removed(queue, result) => { format!("REMOVED {} {}", queue, result)}
          };
          println!("{}", output.as_slice().as_bytes() );
          match write!(buffer, "{}", output) {
            Err(_) => fail!("Failed writing to buffer"),
            _ => {}
          }
          match buffer.flush() {
            Err(_) => { fail!("Failed closing stream") },
            _ => {}
          }
        },
        Err(_) => { fail!("Oha?"); }
      }
    });
  }

  // close the socket server
  drop(acceptor);
}
