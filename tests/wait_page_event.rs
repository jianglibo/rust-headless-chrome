extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use websocket::futures::{Future, Poll, Stream, IntoFuture};
use log::*;
use headless_chrome::browser_async::page_message::{PageMessage};
use headless_chrome::browser_async::debug_session::{DebugSession};
use std::default::Default;
use tokio;


struct LoadEventFired {
    debug_session: DebugSession,
    url: &'static str,
}

impl Future for LoadEventFired {
    type Item = usize;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            info!("my page loop ****************************");
            if let Some(value) = try_ready!(self.debug_session.poll()) {
                match value {
                    PageMessage::EnablePageDone => {
                        info!("page enabled.");
                        self.debug_session.chrome_debug_session.navigate_to(self.url);
                    },
                    PageMessage::SecondsElapsed(seconds) => {
                        if seconds > 39 {
                            break Ok(self.debug_session.chrome_debug_session.changing_frame_tree.child_changing_frames.len().into())
                        }
                        info!("seconds elapsed: {}, page stuck in: {:?} ", seconds, self.debug_session.chrome_debug_session.state);
                    }
                    _ => {
                        info!("got unused page message {:?}", value);
                    }
                }
            } else {
                error!("got None, was stream ended?");
            }
        }
    }
}

#[test]
fn t_load_event_fired() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,browser_async=trace");
    env_logger::init();

    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    let my_page = LoadEventFired {
        debug_session: Default::default(),
        url,
    };
    // tokio::run(my_page.map_err(|e| error!("{:?}", e)));
    // let result = run_one(my_page).unwrap();
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(my_page.into_future()).unwrap();
    assert_eq!(result, 7);
}

// enum Thing {
//     A,
//     B
//     }

//     trait Foo {
//     fn bar(&self) -> usize;
//     }

//     impl Foo for Thing {
//           fn bar(&self) -> usize {
//             match self {
//             Thing::A => 1,
//             Thing::B => 2
//             }
//         }
//     }

    
    // #[test]
    // fn t_vec_to_set() {
    //     let v = vec![1, 2, 3, 3, 4];
    //     let s = v.into_iter().collect::<HashSet<_>>();
    //     assert_eq!(s, [1,2,3,4].iter().cloned().collect());

    //     let a = [&String::from("a"), &String::from("b"), &String::from("c")]; 
    //     let b = ["a", "b", "c", "d"]; 

    //     // assert_eq!(a, b);

    //     // let c: &str = String::from("a").borrow();
    //     // assert_eq!("a", c);

    //     let ai = a.iter().map(|s|&s[..]).collect::<HashSet<_>>();
    //     let bi = b.iter().cloned().collect::<HashSet<_>>();

    //     assert!(ai.is_subset(&bi));
    //     assert!(bi.is_superset(&ai));
    // }