use headless_chrome::browser_async::WaitingForPageAttachTaskName;

use headless_chrome::browser_async::page_message::{MethodCallDone, PageResponse, ReceivedEvent};
use headless_chrome::browser_async::task_describe::{runtime_tasks, HasTaskId};
use headless_chrome::protocol::target;
use log::*;

use super::{GetContentInIframe, HOME_URL, SHENBIAN_GANDONG_URL};

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
                        tab.move_mouse_random_after_secs(2);
                    }
                    // ReceivedEvent::ContentEventFired(_task) => {
                    //     let tab = self
                    //         .get_tab(maybe_target_id)
                    //         .expect("tab should exists. LoadEventFired");
                    //     info!("--------->1 url: {:?}", tab.get_url());
                    // }
                    ReceivedEvent::PageCreated(page_idx) => {
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. PageCreated.");
                        assert!(tab.session_id.is_none());
                        info!("page created: {:?}", tab);
                        tab.attach_to_page();
                    }
                    evv => {
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. LoadEventFired");
                        info!("--------->2 url: {:?}, {:?}", tab.get_url(), evv);
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
                    MethodCallDone::GetProperties(task) => {
                        info!("{:?}", task);
                        assert!(task.task_id_equal(QUERY_ARTICLE_TITLES));
                        // let property_describers = task.task_result.expect("task_result should exists").result;
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. FrameStoppedLoading");

                        let object_id = task
                            .get_array_of_remote_object_id()
                            .get(0)
                            .cloned()
                            .cloned()
                            .expect("object_id should exists.");
                        tab.mouse_click_on_remote_object(object_id);
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
