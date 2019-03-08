pub use std::sync::mpsc::channel;
pub use std::sync::mpsc::Receiver;

struct AselfLifetime {
    an_int: u8
}

impl AselfLifetime {
    fn afn(&'static self) -> Receiver<u8> {
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            tx.send(self.an_int);
        });
        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{stream, Future, Stream, Sink};
    use futures::future::lazy;
    use futures::sync::mpsc;

    #[test]
    fn t_self_lifetime() {
        ::std::env::set_var("RUST_LOG", "headless_chrome=trace,browser_async=debug");
        env_logger::init();
        let a: &'static AselfLifetime = &AselfLifetime {an_int: 3};
        assert_eq!(a.afn().recv().unwrap(), 3);
    }

    #[test]
    fn t_spwan_future() {
        fn a_future() -> impl Future<Item=u8, Error=failure::Error> {
            futures::future::ok(1_u8)
        }

        // tokio::run(futures::lazy(|| {
            tokio::spawn(a_future().map_err(|_| println!("error")).map(|_| println!("One")));
            // Ok(())
        // }));
    }
    #[test]
    fn t_standard_futrue() {
        tokio::run(lazy(|| {
            let (tx, rx) = mpsc::channel(1_024);

            tokio::spawn({
                stream::iter_ok(0..10).fold(tx, |tx, i| {
                    tx.send(format!("Message {} from spawned task", i))
                        .map_err(|e| println!("error = {:?}", e))
                })
                .map(|_| ()) // Drop tx handle
            });

            

            rx.for_each(|msg| {
                println!("Got `{}`", msg);
                Ok(())
            })
        }));
    }
}
