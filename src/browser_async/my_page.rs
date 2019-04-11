use crate::protocol;
use crate::protocol::page;
use crate::protocol::dom;
use crate::browser_async::chrome_browser::{ChromeBrowser};
use crate::browser_async::dev_tools_method_util::{
    MethodUtil,MethodDestination, MethodBeforSendResult, ChromePageError,
};
use crate::protocol::page::methods::{Navigate};
use crate::protocol::target;
use websocket::futures::{Async, Future, Poll, Stream, IntoFuture};
use log::*;
use super::one_page::{OnePage};
use super::page_message::{PageMessage, PageEventName, ChangingFrameTree, ChangingFrame};
use super::interval_one_page::{IntervalOnePage};
use std::fs;
use std::time::{Duration, Instant};
use tokio::timer::{Interval};
use tokio_timer::*;
use tokio::prelude::stream::Select;

trait WaitMaxSeconds {
    fn get_max_wait_seconds(&self) -> usize;

    fn panic_if_timeout(&self, seconds: usize) -> ChromePageError {
        ChromePageError::WaitTimeout{seconds: self.get_max_wait_seconds()}
    }
}


pub struct LoadEventFired {
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


pub struct FindNode {
    chrome_page: IntervalOnePage,
    url: &'static str,
    selector: &'static str,
    root_node_id: Option<dom::NodeId>,
}

impl Future for FindNode {
    type Item = Option<dom::NodeId>;
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
                        info!("seconds elipsed: {}, page stuck in: {:?} ", seconds, self.chrome_page.one_page.state);
                        if seconds > 39 {
                            error!("time out {}", seconds);
                            panic!("time out 40 seconds.");
                        }
                    }
                    // PageMessage::PageEvent(PageEventName::loadEventFired) => {
                    PageMessage::PageEvent(_) => {
                        if let Some(_) = self.chrome_page.one_page.is_frame_navigated("ddlogin-iframe") {
                            self.chrome_page.one_page.dom_query_selector_by_selector(self.selector, Some(5));
                        }
                    }
                    PageMessage::NodeIdComing(node_id, task) => {
                        info!("task done: {:?}", task);
                        break Ok(Some(node_id).into());
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

// pub struct DescribeNode {
//     chrome_page: IntervalOnePage,
//     url: &'static str,
//     selector: &'static str,
// }

// impl Future for DescribeNode {
//     type Item = Option<dom::Node>;
//     type Error = failure::Error;

//     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//         loop {
//             info!("my page loop ****************************");
//             if let Some(value) = try_ready!(self.chrome_page.poll()) {
//                 match value {
//                     PageMessage::EnablePageDone => {
//                         info!("page enabled.");
//                         self.chrome_page.one_page.navigate_to(self.url);
//                     },
//                     PageMessage::SecondsElapsed(seconds) => {
//                         info!("seconds elipsed: {}, page stuck in: {:?} ", seconds, self.chrome_page.one_page.state);
//                     }
//                     PageMessage::FrameNavigatedEvent(session_id, target_id, frame_navigated_event) => {
//                         info!("frame event: {:?}", frame_navigated_event);
//                         if let Some(frame_name) = frame_navigated_event.params.frame.name {
//                             if frame_name == "ddlogin-iframe" {
//                                 self.chrome_page.one_page.dom_describe_node(Some(self.selector), None, false);
//                             }
//                         }
//                     }
//                     PageMessage::DomDescribeNode(selector, node) => {
//                         break Ok(node.into());
//                     }
//                     _ => {
//                         info!("got unused page message {:?}", value);
//                     }
//                 }
//             } else {
//                 error!("got None, was stream ended?");
//             }
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use websocket::futures::{Future};
    use futures::{task};
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::borrow::Borrow;

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

    fn init_log() {
        ::std::env::set_var("RUST_LOG", "headless_chrome=info,browser_async=trace");
        env_logger::init();
    }

    fn get_fixture_page() -> IntervalOnePage {
        let browser = ChromeBrowser::new();
        let page = OnePage::new(browser);
        IntervalOnePage::new(page)
    }

        enum Thing {
            A,
            B
            }

            trait Foo {
            fn bar(&self) -> usize;
            }

            impl Foo for Thing {
                  fn bar(&self) -> usize {
                    match self {
                    Thing::A => 1,
                    Thing::B => 2
                    }
                }
            }

    #[test]
    fn t_vec_to_set() {
        let v = vec![1, 2, 3, 3, 4];
        let s = v.into_iter().collect::<HashSet<_>>();
        assert_eq!(s, [1,2,3,4].iter().cloned().collect());

        let a = [&String::from("a"), &String::from("b"), &String::from("c")]; 
        let b = ["a", "b", "c", "d"]; 

        // assert_eq!(a, b);

        // let c: &str = String::from("a").borrow();
        // assert_eq!("a", c);

        let ai = a.iter().map(|s|&s[..]).collect::<HashSet<_>>();
        let bi = b.iter().cloned().collect::<HashSet<_>>();

        assert!(ai.is_subset(&bi));
        assert!(bi.is_superset(&ai));

    }

    #[test]
    fn t_load_event_fired() {
        init_log();
        let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
        let my_page = LoadEventFired {
            chrome_page: get_fixture_page(),
            url,
        };
        // tokio::run(my_page.map_err(|e| error!("{:?}", e)));
        let result = run_one(my_page).unwrap();
        assert_eq!(result, 7);
    }

    #[test]
    fn t_dom_query_selector() {
        init_log();
        let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
        let mut selector = "#ddlogin-iframe #qrcode";
        let my_page = FindNode {
            chrome_page: get_fixture_page(),
            url,
            selector,
            root_node_id: None,
        };
        // tokio::run(my_page.map_err(|e| error!("{:?}", e)));
        // let result = run_one(my_page).unwrap();
        // assert!(result.is_none());

        selector = "#ddlogin-iframe";
        let my_page = FindNode {
            chrome_page: get_fixture_page(),
            url,
            selector,
            root_node_id: None,
        };
        // tokio::run(my_page.map_err(|e| error!("{:?}", e)));
        let result = run_one(my_page).unwrap();
        assert!(result.is_some());
    }
    // #[test]
    // fn t_dom_describe_node() {
    //     init_log();
    //     let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    //     let mut selector = "#ddlogin-iframe #qrcode";
    //     let my_page = DescribeNode {
    //         chrome_page: get_fixture_page(),
    //         url,
    //         selector,
    //     };
    //     // tokio::run(my_page.map_err(|e| error!("{:?}", e)));
    //     let result = run_one(my_page).unwrap();
    //     assert!(result.is_none());

    //     selector = "#ddlogin-iframe";
    //     let my_page = DescribeNode {
    //         chrome_page: get_fixture_page(),
    //         url,
    //         selector,
    //     };
    //     // tokio::run(my_page.map_err(|e| error!("{:?}", e)));
    //     let result = run_one(my_page).unwrap();
    //     assert!(result.is_some());

    //     let node = result.unwrap();

    //     assert_eq!(node.node_id, 0); // describe node doesn't return node_id again.
    //     assert!(node.backend_node_id > 0);
    //     assert!(node.children.unwrap().len() == 0);
    //     assert_eq!(node.parent_id, None);
    //     assert_eq!(node.node_value, "");
    //     assert_eq!(node.node_name, "IFRAME");
    //     assert_eq!(node.node_type, 1);
    //     assert_eq!(node.local_name, "iframe");
    //     let attrs: HashMap<String, String> = node.attributes.unwrap();
    //     assert_eq!(attrs.get("ddlogin-iframe"), None);

    //     let content_document = node.content_document.unwrap();
    //     assert_eq!(content_document.node_name, "#document");
    //     assert!(content_document.backend_node_id > 0);

    // }
}
