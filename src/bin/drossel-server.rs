#![crate_type = "bin"]
#![feature(globs,phase)]
//#![phase(syntax, link)] extern crate log;

extern crate green;
extern crate rustuv;
extern crate drossel;
extern crate strand;

use std::io::net::tcp::{TcpListener,TcpStream};
use std::io::{Acceptor, Listener};
use std::io::{BufferedStream,Stream};
use drossel::commands::util::get_command;
use drossel::commands::command::Command;
use drossel::drossel::types::{DBResult,Pong,Inserted,Removed};
use drossel::drossel::db::{DB};
use drossel::drossel::store::*;
use strand::mutable::{Event,AsSendableEvent};
use strand::errors::{Errors};

type DBEnvelope = Envelope<Box<Event<BinaryList, DBResult>+Send>,Result<DBResult, Errors>>;

#[deriving(Send)]
struct Envelope<M: Send, R: Send> {
  message: M,
  reply_to: Sender<R>
}

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    green::start(argc, argv, rustuv::event_loop, main)
}

fn main() {
  let db_sender = start_db();

  let listener = TcpListener::bind("127.0.0.1", 7890).unwrap();

  tcp_handler(listener, db_sender);
}

fn handle_stream<T: Stream>(
    queue: Sender<DBEnvelope>,
    mut buffer: BufferedStream<T>
) {
  let input = buffer.read_until('\n' as u8).unwrap();

  let command: Box<Command> = get_command(input).unwrap();
  let (sender, receiver) = channel::<Result<DBResult, Errors>>();
  let event = (*command).as_sendable_event();
  queue.send(Envelope { message: event, reply_to: sender });

  let res = receiver.recv().unwrap();
  let output = match res {
    Pong                   => format!("PONG"),
    Inserted(queue)        => format!("OK {}", queue),
    Removed(queue, result) => format!("REMOVED {} {}", queue, result)
  };

  match write!(buffer, "{}", output) {
    Err(e) => fail!("Failed writing to buffer: {}", e),
    _ => {}
  }
  match buffer.flush() {
    Err(e) => fail!("Failed flushing buffer: {}", e),
    _ => {}
  }
}

fn start_db() -> Sender<DBEnvelope> {
  let (sender, receiver) = channel::<DBEnvelope>();
  // spawn the db task
  spawn(proc() {
    let mut db = DB::new();
    for m in receiver.iter() {
      m.reply_to.send(db.execute(m.message));
    }
  });

  sender
}

fn tcp_handler(
    listener: TcpListener,
    queue: Sender<DBEnvelope>,
) {
    let mut acceptor = listener.listen();

    for stream in acceptor.incoming() {
        match stream {
            Err(_) => fail!("connection error!"),
            Ok(conn) => {
                handler_task(&queue, conn);
            }
        }
    }

    drop(acceptor);
}

fn handler_task(sender: &Sender<DBEnvelope>, conn: TcpStream) {
  let cloned_sender = sender.clone();

  spawn(proc() {
      handle_stream(
          cloned_sender,
          BufferedStream::new(conn)
      );
  });
}
