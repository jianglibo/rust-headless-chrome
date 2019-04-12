extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::protocol::{self, page, dom};
use headless_chrome::browser_async::chrome_browser::{ChromeBrowser};
use headless_chrome::browser_async::dev_tools_method_util::{
    MethodUtil,MethodDestination, MethodBeforSendResult, ChromePageError,
};

use websocket::futures::{Async, Future, Poll, Stream, IntoFuture};
use log::*;
use headless_chrome::browser_async::one_page::{OnePage};
use headless_chrome::browser_async::page_message::{PageMessage, PageEventName, ChangingFrameTree, ChangingFrame};
use headless_chrome::browser_async::interval_one_page::{IntervalOnePage};
use std::fs;
use std::time::{Duration, Instant};
use tokio::timer::{Interval};
use tokio_timer::*;
use tokio::prelude::stream::Select;
use tokio;
use futures::{task};
use std::collections::HashMap;
use std::collections::HashSet;
use std::borrow::Borrow;


struct LoadEventFired {
    chrome_page: IntervalOnePage,
    url: &'static str,
}

impl Future for LoadEventFired {
    type Item = usize;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            info!("my page loop ****************************");
            if let Some(value) = try_ready!(self.chrome_page.poll()) {
                match value {
                    PageMessage::EnablePageDone => {
                        info!("page enabled.");
                        self.chrome_page.one_page.navigate_to(self.url);
                    },
                    PageMessage::SecondsElapsed(seconds) => {
                        if seconds > 39 {
                            break Ok(self.chrome_page.one_page.changing_frame_tree.child_changing_frames.len().into())
                        }
                        info!("seconds elipsed: {}, page stuck in: {:?} ", seconds, self.chrome_page.one_page.state);
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

fn run_one<F>(f: F) -> Result<F::Item, F::Error>
where
    F: IntoFuture,
    F::Future: Send + 'static,
    F::Item: Send + 'static,
    F::Error: Send + 'static,
{
        let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        runtime.block_on(f.into_future())
}

fn get_fixture_page() -> IntervalOnePage {
    let browser = ChromeBrowser::new();
    let page = OnePage::new(browser);
    IntervalOnePage::new(page)
}

#[test]
fn t_load_event_fired() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,browser_async=trace");
    env_logger::init();

    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    let my_page = LoadEventFired {
        chrome_page: get_fixture_page(),
        url,
    };
    // tokio::run(my_page.map_err(|e| error!("{:?}", e)));
    let result = run_one(my_page).unwrap();
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