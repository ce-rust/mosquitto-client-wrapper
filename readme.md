# A Rust interface to the Mosquitto MQTT broker client

![build](https://github.com/ce-rust/mosquitto-client-wrapper/workflows/build/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/mosquitto-client-wrapper)](https://docs.rs/crate/mosquitto-client-wrapper/*)

Mosquitto is a popular MQTT broker implemented in C. Although there are pure
Rust MQTT clients, it is still useful to have a binding to the Mosquitto client.

The basic story is that you connect to a broker, _subscribing_ to topics that
interest you and _publishing_ messages on a particular topic. The messages
may be any arbitrary bytes, but this implementation does require that the topics
themselves be UTF-8.  The C API is based on callbacks, which are mapped onto
Rust closures.

The Mosquitto client is thread-safe, so you can publish from one thread and listen
for the messages on another. This example demonstrates **mosquitto-client** usage:

```rust
extern crate mosquitto_client_wrapper as mosq;
use mosq::Mosquitto;
use std::{thread, time};

fn main() {
    let m = Mosquitto::new("test");

    m.connect("localhost",1883,5).expect("can't connect");
    m.subscribe("bilbo/#",1).expect("can't subscribe to bonzo");

    let mt = m.clone();
    thread::spawn(move || {
        let timeout = time::Duration::from_millis(500);
        for _ in 0..5 {
            mt.publish("bilbo/baggins","hello dolly".as_bytes(), 1, false).unwrap();
            thread::sleep(timeout);
        }
        mt.disconnect().unwrap();
    });

    let mut mc = m.callbacks(0);
    mc.on_message(|data,msg| {
        println!("bilbo {:?}",msg);
        *data += 1;
    });

    m.loop_until_disconnect(200).expect("broken loop");
    println!("received {} messages",mc.data);
}
```
The `Mosquitto` struct is a thin wrapper around the C pointer we get from the client.
It is `Clone + Send + Sync`, so we can pass it to a thread which simply publishes some
bytes and waits; at the end of the thread we _explicitly_ disconnect.

The ``Callbacks`` handler struct is separate, to avoid antagonizing the borrow checker.
It is created by the `callbacks` method and is generic over some data (accessed as the `data` field)
Whenever an event occurs, the callback will be passed a mutable reference to that data, and
event-specific data - in this case a message struct.

`loop_until_disconnect` is a relative of `loop_forever` which ends without error if we
explicitly disconnect from the broker.

## Features

### bundled (on by default)

This feature will automatically download and build the mosquitto client library.
By default, the master branch is taken. This could be overwritten with the following variables:

*  `MOSQUITTO_GIT_URL` Url of the git repository; by default [https://github.com/eclipse/mosquitto.git](https://github.com/eclipse/mosquitto.git)
*  `MOSQUITTO_GIT_HASH` Git hash to checkout for the build; by default none

#### Cross compilation

The library could be cross-compiled. The following env parameters have to be set for that:

*  `MOSQUITTO_CROSS_COMPILER` default none, e.g. `arm-linux-gnueabihf-`
*  `MOSQUITTO_CC` default `gcc`
