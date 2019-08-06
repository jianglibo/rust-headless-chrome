use headless_chrome::browser_async::page_message::{
    write_base64_str_to, MethodCallDone, PageResponse, ReceivedEvent,
};
use headless_chrome::browser_async::task_describe::{runtime_tasks, HasTaskId, TaskDescribe};
use headless_chrome::protocol::{page, target};
use log::*;
use std::fs;
use std::path::Path;
use rand::prelude::*;
use rand::seq::SliceRandom;

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

    pub fn home_page_full_displayed(
        &mut self,
        maybe_target_id: Option<&target::TargetId>,
        page_response: PageResponse,
    ) {
        // let expression = r##"document.querySelectorAll('#\\32 31c div.grid-cell span.text').length"##;
        let shenbian_gandong_task_name = "shenbian-gandong";
        match page_response {
            PageResponse::ReceivedEvent(received_event) => {
                match received_event {
                    ReceivedEvent::FrameStoppedLoading(_frame_id) => {
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. FrameStoppedLoading");

                        info!(
                            "************frames: {:?}, tab: {:?}, frams: {:?}",
                            tab.changing_frames.len(),
                            tab.get_url(),
                            tab.changing_frames
                        );
                        if tab.get_url().contains("/lgpage/detail/")
                            && tab.changing_frames.count_stopped() == 2
                        {
                            info!("*r* detail_page");
                            tab.display_full_page_after_secs(2);
                            tab.set_move_mouse_random_interval(20, 40);
                        }
                    }
                    // ReceivedEvent::LifeCycle => {
                    //     let tab = self
                    //         .get_tab(maybe_target_id)
                    //         .expect("tab should exists. LoadEventFired");
                    //     info!("kkkkkkkkkkkkkkkkkkkkkkkkkkkkk: {:?}", tab.life_cycles);

                    //     if tab.get_url().contains("/lgpage/detail/")
                    //         && tab.life_cycles.last_life_cycle_event().is_network_almost_idle()
                    //     {
                    //         tab.display_full_page();
                    //         tab.set_move_mouse_random_interval(20, 40);
                    //     }
                    //     info!("got lifecycleEvent, home_page_full_displayed: {:?}", tab.life_cycles.last_life_cycle_event());
                    // }
                    ReceivedEvent::PageCreated => {
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. PageCreated.");
                        assert!(tab.session_id.is_none());
                        info!("page created: {:?}", tab);
                        // tab.name_the_page(DETAIL_PAGE);
                        tab.page_enable();
                        tab.runtime_enable();
                        // tab.network_enable();
                        tab.log_enable();
                        // tab.set_move_mouse_random_interval(8, 20);
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
                        }
                    }
                    MethodCallDone::GetProperties(task) => {
                        info!("{:?}", task);
                        assert!(task.task_id_equal(QUERY_ARTICLE_TITLES));
                        // let property_describers = task.task_result.expect("task_result should exists").result;
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. FrameStoppedLoading");

                        // let object_id =
                        let mut tasks: Vec<Vec<TaskDescribe>> = task
                            .get_array_of_remote_object_id()
                            .iter()
                            .map(|&oid| oid.to_string())
                            .map(|oid| tab.mouse_click_on_remote_object_task(oid))
                            .collect();
                        
                        let mut rng = rand::thread_rng();
                        tasks.shuffle(&mut rng);
                        tab.execute_task_vecs_manually_later(tasks);
                        // tab.execute_task_vecs_in_interval(tasks.drain(..1).collect(), 10);
                        // .get(0)
                        // .cloned()
                        // .cloned()
                        // .expect("object_id should exists.");
                        // tab.mouse_click_on_remote_object(object_id);
                    }
                    MethodCallDone::GetTargets(task) => {
                        info!("**GetTargets: {:?}", task.task_result);
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
