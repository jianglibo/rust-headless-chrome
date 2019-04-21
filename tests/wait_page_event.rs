extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use websocket::futures::{Future, Poll, Stream, IntoFuture};
use log::*;
use headless_chrome::browser_async::page_message::{PageResponse};
use headless_chrome::browser_async::debug_session::{DebugSession};
use headless_chrome::browser_async::page_message::{ChangingFrame};
use std::default::Default;
use tokio;


struct LoadEventFired {
    debug_session: DebugSession,
    url: &'static str,
    root_node: Option<u16>,
}

impl Future for LoadEventFired {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some((tab_id, _task_id, value)) = try_ready!(self.debug_session.poll()) {
                let tab = if let Some(tid) = &tab_id {
                    self.debug_session.get_tab_by_id_mut(tid)
                } else {
                    None
                };
                match value {
                    PageResponse::PageEnable => {
                        info!("page enabled.");
                        tab.unwrap().navigate_to(self.url);
                    },
                    PageResponse::FrameNavigated(changing_frame) => {
                        info!("got frame: {:?}", changing_frame);
                        if let ChangingFrame::Navigated(frame) = changing_frame {
                            if frame.name == Some("ddlogin-iframe".into()) {
                                info!("send get document command.");
                                tab.unwrap().get_document(None, Some(100));
                            }
                        }
                    }
                    PageResponse::GetDocument => {
                        if let Some(nd) = &tab.unwrap().root_node {
                            self.root_node = Some(nd.node_id);
                        }
                        
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        info!("seconds elapsed: {} ", seconds);
                        if seconds > 29 {
                            let tab = self.debug_session.main_tab_mut().unwrap();
                            assert_eq!(tab.changing_frames.len(), 7);
                            if let Some(frame) = tab.main_frame() {
                                assert_eq!(tab.target_info.target_id, frame.id);
                            } else {
                                panic!("test failed.");
                            }
                            assert!(self.root_node.is_some());
                            // assert!(tab.temporary_node_holder.len() > 2);
                            break Ok(().into())
                        }
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
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,wait_page_event=trace");
    env_logger::init();

    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    let my_page = LoadEventFired {
        debug_session: Default::default(),
        url,
        root_node: None,
    };
    // tokio::run(my_page.map_err(|e| error!("{:?}", e)));
    // let result = run_one(my_page).unwrap();
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).unwrap();
    // assert!(my_page.root_node.is_some());
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