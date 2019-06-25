use headless_chrome::browser_async::{WaitingForPageAttachTaskName};

use headless_chrome::browser_async::page_message::{
    MethodCallDone, PageResponse, ReceivedEvent,
};
use headless_chrome::browser_async::task_describe::{HasTaskId, dom_tasks, runtime_tasks};
use headless_chrome::protocol::target;
use log::*;

use super::{HOME_URL, GetContentInIframe, SHENBIAN_GANDONG_URL};

const QUERY_ARTICLE_TITLES: &str = "query-article-titles";
const DESCRIBE_ARTICLE_TITLES: &str = "describe-article-titles";

impl GetContentInIframe {
    pub fn waiting_for_qrcode_scan(
        &mut self,
        maybe_target_id: Option<&target::TargetId>,
        page_response: PageResponse,
    ) {
        let expression = r##"document.querySelectorAll('#\\32 31c div.grid-cell span.text').length"##;
        let get_children_number_task_name = "get-children-number";
        let shenbian_gandong_task_name = "shenbian-gandong";
        let shiping_children_num_task_name = "shiping-children-num";
        match page_response {
            PageResponse::ReceivedEvent(received_event) => {
                match received_event {
                    ReceivedEvent::PageCreated(_page_idx) => {
                        let tab = self.get_tab(maybe_target_id).expect("tab should exists.");
                        let tasks = vec![
                            WaitingForPageAttachTaskName::PageEnable,
                            WaitingForPageAttachTaskName::RuntimeEnable,
                            // WaitingForPageAttachTaskName::NetworkEnable
                            ];
                        tab.attach_to_page_and_then(tasks);
                    }
                    ReceivedEvent::FrameStoppedLoading(_frame_id) => {
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. FrameStoppedLoading");
                        info!("url current: {:?}", tab.get_url());
                        if tab.is_at_url(HOME_URL) {
                            tab.explicitly_close = true;
                            tab.name_the_page(HOME_URL);
                            let task = tab.evaluate_expression_task_named(expression, get_children_number_task_name);
                            tab.task_queue.add_delayed(task, 2);
                            self.debug_session.create_new_tab_named(SHENBIAN_GANDONG_URL, shenbian_gandong_task_name);
                        } else if tab.is_at_url(SHENBIAN_GANDONG_URL) {
                            tab.explicitly_close = true;
                            tab.name_the_page(SHENBIAN_GANDONG_URL);
                            let task = tab.evaluate_expression_task_named(r##"document.querySelectorAll("#root div.grid-cell span.text").length"##, shiping_children_num_task_name);
                            tab.task_queue.add_delayed(task, 3);
                        }
                    }
                    ReceivedEvent::ResponseReceived(_event) => {}
                    _ => {
                        // info!("got unused page event {:?}", received_event);
                    }
            }
            }
            PageResponse::MethodCallDone(method_call_done) => {
                match method_call_done {
                    MethodCallDone::Evaluate(task) => {
                        info!("{:?}", task);
                        
                        if task.task_id_equal(get_children_number_task_name) {
                            if let Some(v) = task.get_u64_result() {
                                assert!(v == 16);
                                let tab = self
                                    .get_tab(maybe_target_id)
                                    .expect("tab should exists. FrameStoppedLoading");
                                let children_nodes_expression = r##"document.querySelectorAll('#\\32 31c div.grid-cell span.text')"##;
                                tab.evaluate_expression_named(children_nodes_expression, QUERY_ARTICLE_TITLES);
                                // let fm = |i: u64| {
                                //     format!(r##"document.querySelectorAll('#\\32 31c div.grid-cell span.text').item({}).click()"##, i)
                                // };
                                // for i in 0..15 {
                                //     let exp = fm(i);
                                //     let slice = exp.as_str();
                                //     let t1 = tab.evaluate_expression_task(slice);
                                //     tab.task_queue.add_manually(t1);
                                // }
                            } else {
                                panic!("unexpected call return.");
                            }
                        } else if task.task_id_equal(shiping_children_num_task_name) {
                            if let Some(v) = task.get_u64_result() {
                                error!("vvvvvvvvvvvvvvvvvvvvvvvvvvv{:?}",v);
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
                        } else if task.task_id_equal(QUERY_ARTICLE_TITLES) {
                                let tab = self
                                    .get_tab(maybe_target_id)
                                    .expect("tab should exists. FrameStoppedLoading");
                                let remote_object_id = task.get_object_id().expect("remote_object_id should exists.");
                                tab.get_properties_by_object_id_named(remote_object_id, DESCRIBE_ARTICLE_TITLES);
                        } else {
                            info!("{:?}", task);
                        }
                    } 
                    MethodCallDone::GetProperties(task) => {
                        info!("{:?}", task);
                        assert!(task.task_id_equal(DESCRIBE_ARTICLE_TITLES));
                        let property_describers = task.task_result.expect("task_result should exists").result;
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. FrameStoppedLoading");
                        // for pd in property_describers {
                        //     if let Some(ro) = pd.value {
                        //         if let Some(object_id) = ro.object_id {
                        //             info!("{:?}", object_id);
                        //             let tasks = tab.mouse_click_on_remote_object_task(object_id);
                        //             tab.task_queue.add_manually_many(tasks);
                        //         }
                        //     }
                        // }
                        // tab.run_task_queue_manually();
                        if let Some(pd) = property_describers.get(0) {
                            if let Some(ro) = &pd.value {
                                if let Some(object_id) = &ro.object_id {
                                    info!("{:?}", object_id);
                                    let task = tab.get_content_quads_by_object_id_task_named(object_id.to_string(), "get-quads");
                                    tab.execute_one_task(task);

                                    let task = tab.get_js_midpoint_task(object_id.to_string(), Some("get-js-midpoint"));
                                    tab.execute_one_task(task);
                                    // let tasks = tab.mouse_click_on_remote_object_task(object_id.to_string());
                                    // tab.execute_tasks(tasks);
                                }
                            }
                        } else {
                            panic!("empty property_describers.");
                        }
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
                // info!("{:?}", method_call_done);
            }
            _ => {}
        }
    }
}