use headless_chrome::browser_async::page_message::{MethodCallDone, PageResponse, ReceivedEvent, write_base64_str_to};
use headless_chrome::browser_async::task_describe::{runtime_tasks, HasTaskId};
use headless_chrome::protocol::{target, page};
use std::fs;
use std::path::Path;
use log::*;

use super::{GetContentInIframe, HOME_URL, SHENBIAN_GANDONG_URL, DETAIL_PAGE};

const QUERY_ARTICLE_TITLES: &str = "query-article-titles";
const SHIPING_CHILDREN_NUM_TASK_NAME: &str = "shiping-children-num";

impl GetContentInIframe {
    fn handle_evaluate(
        &mut self,
        task: runtime_tasks::EvaluateTask,
        maybe_target_id: Option<&target::TargetId>,
    ) {
        info!("{:?}", task);
        // if task.task_id_equal(GET_CHILDREN_NUMBER_TASK_NAME) {
        //     if let Some(v) = task.get_u64_result() {
        //         assert!(v == 16);
        //         let tab = self
        //             .get_tab(maybe_target_id)
        //             .expect("tab should exists. FrameStoppedLoading");
        //         let children_nodes_expression = r##"document.querySelectorAll('#\\32 31c div.grid-cell span.text')"##;
        //         tab.evaluate_expression_named(children_nodes_expression, QUERY_ARTICLE_TITLES);
        //         // let fm = |i: u64| {
        //         //     format!(r##"document.querySelectorAll('#\\32 31c div.grid-cell span.text').item({}).click()"##, i)
        //         // };
        //         // for i in 0..15 {
        //         //     let exp = fm(i);
        //         //     let slice = exp.as_str();
        //         //     let t1 = tab.evaluate_expression_task(slice);
        //         //     tab.task_queue.add_manually(t1);
        //         // }
        //     } else {
        //         panic!("unexpected call return.");
        //     }
        // } else
        if task.task_id_equal(SHIPING_CHILDREN_NUM_TASK_NAME) {
            if let Some(v) = task.get_u64_result() {
                error!("vvvvvvvvvvvvvvvvvvvvvvvvvvv{:?}", v);
                assert!(v > 0);
                let tab = self
                    .get_tab(maybe_target_id)
                    .expect("tab should exists. FrameStoppedLoading");
                let fm = |i: u64| {
                    format!(r##"document.querySelectorAll("#root div.grid-cell span.text").item({}).click()"##, i)
                };
                for i in 0..15 {
                    let exp = fm(i);
                    let slice = exp.as_str();
                    let t1 = tab.evaluate_expression_task(slice);
                    tab.task_queue.add_manually(t1);
                }
            } else {
                panic!("unexpected call return.");
            }
        } else {
            info!("{:?}", task);
        }
    }

    pub fn home_page_full_displayed (
        &mut self,
        maybe_target_id: Option<&target::TargetId>,
        page_response: PageResponse,
    ) {
        // let expression = r##"document.querySelectorAll('#\\32 31c div.grid-cell span.text').length"##;
        let shenbian_gandong_task_name = "shenbian-gandong";
        match page_response {
            PageResponse::ReceivedEvent(received_event) => {
                match received_event {
                    ReceivedEvent::LoadEventFired(_task) => {
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. LoadEventFired");
                        info!("---------> url: {:?}", tab.get_url());
                        assert!(tab.get_url().contains("/lgpage/detail/"));
                        assert_eq!(tab.page_name, Some(DETAIL_PAGE));
                        // let tt = tab.mouse_move_to_xy_task(101.0, 101.0);
                        // tab.execute_tasks_after_secs(vec![tt], 66);
                        // tab.move_mouse_random_after_secs(60);
                        // let tasks = vec![tab.capture_screenshot_jpeg_task(Some(100), None, Some("target/gcii.jpeg"))];
                        let mut tasks = tab.display_full_page_task();
                        tasks.push(tab.capture_screenshot_jpeg_task(Some(100), None, Some("target/gcii.jpeg")));
                        tab.execute_tasks_after_secs(tasks, 6);
                        tab.activate_page();
                        tab.set_move_mouse_random_interval(8, 20);
                    }
                    ReceivedEvent::PageCreated => {
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. PageCreated.");
                        assert!(tab.session_id.is_none());
                        info!("page created: {:?}", tab);
                        tab.name_the_page(DETAIL_PAGE);
                        tab.page_enable();
                        tab.runtime_enable();
                        tab.network_enable();
                        tab.attach_to_page();
                    }
                    _evv => {
                        // let tab = self
                        //     .get_tab(maybe_target_id)
                        //     .expect("tab should exists. LoadEventFired");
                        // info!("--------->2 url: {:?}, {:?}", tab.get_url(), evv);
                    }
                }
            }
            PageResponse::MethodCallDone(method_call_done) => {
                match method_call_done {
                    MethodCallDone::Evaluate(task) => {
                        self.handle_evaluate(task, maybe_target_id);
                    }
                    MethodCallDone::SetDeviceMetricsOverride(_task) => {
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. SetDeviceMetricsOverride");
                        info!("url current: {:?}", tab.get_url());
                        if tab.is_at_url(HOME_URL) {
                            tab.explicitly_close = true;
                            tab.name_the_page(HOME_URL);
                            let children_nodes_expression = r##"document.querySelectorAll('#\\32 31c div.grid-cell span.text')"##;
                            tab.evaluate_expression_and_get_properties_named(
                                children_nodes_expression,
                                QUERY_ARTICLE_TITLES,
                            );
                        } else if tab.is_at_url(SHENBIAN_GANDONG_URL) {
                            tab.explicitly_close = true;
                            tab.name_the_page(SHENBIAN_GANDONG_URL);
                            let task = tab.evaluate_expression_task_named(r##"document.querySelectorAll("#root div.grid-cell span.text").length"##, SHIPING_CHILDREN_NUM_TASK_NAME);
                            tab.execute_one_task(task);
                            tab.evaluate_expression_named(r##"document.hidden"##, "dh");
                            // tab.task_queue.add_delayed(task, 3);
                        }
                    }

                    MethodCallDone::CaptureScreenshot(capture_screen_shot) => {
                        // info!("got screen shot: {:?}", capture_screen_shot.task_result);
                        info!("got screen shot: >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
                        assert!(capture_screen_shot.task_result.is_some());
                        let path = Path::new("target/gcii.jpeg");
                        assert!(path.exists());
                    }
                    MethodCallDone::GetProperties(task) => {
                        info!("{:?}", task);
                        assert!(task.task_id_equal(QUERY_ARTICLE_TITLES));
                        // let property_describers = task.task_result.expect("task_result should exists").result;
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. FrameStoppedLoading");

                        // let object_id = 
                        task
                            .get_array_of_remote_object_id()
                            .iter()
                            .map(|&oid|oid.to_string())
                            .for_each(|oid|{
                                let task = tab.mouse_click_on_remote_object_task(oid);
                                tab.execute_task_manually_later(task);
                            });
                            // .get(0)
                            // .cloned()
                            // .cloned()
                            // .expect("object_id should exists.");
                        // tab.mouse_click_on_remote_object(object_id);
                    }
                    MethodCallDone::GetContentQuads(task) => {
                        info!("-------------{:?}", task);
                    }
                    MethodCallDone::CallFunctionOn(task) => {
                        info!("---------------{:?}", task);
                    }
                    MethodCallDone::GetResponseBodyForInterception(_task) => {}
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
