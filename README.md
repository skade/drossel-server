# drossel-server

Drossel is a (very unfinished) queue server modeled after [Kestrel](http://github.com/twitter/kestrel).

## Prerequisites

[Cargo](http://crates.io) is required to build. To install Cargo and Rust in one go, use:

```sh
$ curl https://static.rust-lang.org/rustup.sh | sudo bash
```

## Compilation

Compile using:

```sh
$ cargo build
```

## Run

Run using:

```sh
$ target/drossel-server
```

The server runs on port 7890.

## Interact

drossel currently only understands three messages, delivered over a raw TCP connection:

Set an item on a queue:

```
SET <queue_name> binary
```

Get an item from a queue:

```
GET <queue_name>
```

The <queue_name> is currently ignored (read the unfinished thing? ;)).

## TODO

* Give it a license
* Implement persistence
* Implement multiple queues
* Implement fan-out queues
* Implement tenative fetch

