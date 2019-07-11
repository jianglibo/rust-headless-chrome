use super::Tab;
use super::super::protocol::{page, runtime, target, input};
use super::{TaskId, ChromeDebugSession};
use super::super::task_describe::{
    dom_tasks, page_tasks, runtime_tasks, target_tasks,
    CommonDescribeFields, CommonDescribeFieldsBuilder, TaskDescribe, input_tasks,
};

impl Tab {
    fn get_box_model_task_impl(
        &mut self,
        mut get_box_model_task_builder: dom_tasks::GetBoxModelTaskBuilder,
        manual_task_id: Option<&str>,
    ) -> TaskDescribe {
        let task = get_box_model_task_builder
            .common_fields(self.get_common_field(manual_task_id.map(Into::into)))
            .build()
            .expect("GetBoxModelTaskBuilder should success.");
        task.into()
    }
    pub fn get_box_model_task(
        &mut self,
        get_box_model_task_builder: dom_tasks::GetBoxModelTaskBuilder,
    ) -> TaskDescribe {
        self.get_box_model_task_impl(get_box_model_task_builder, None)
    }

    pub fn get_box_model_task_named(
        &mut self,
        get_box_model_task_builder: dom_tasks::GetBoxModelTaskBuilder,
        name: &str,
    ) -> TaskDescribe {
        self.get_box_model_task_impl(get_box_model_task_builder, Some(name))
    }

    pub fn get_box_model_by_selector_task(&self, selector: &str) -> Vec<TaskDescribe> {
        self.get_box_model_by_selector_task_impl(selector, None)
    }

    pub fn get_box_model_by_selector_task_named(&self, selector: &str, name: &str) -> Vec<TaskDescribe> {
        self.get_box_model_by_selector_task_impl(selector, Some(name))
    }


    fn get_box_model_by_selector_task_impl(
        &self,
        selector: &str,
        manual_task_id: Option<&str>,
    ) -> Vec<TaskDescribe> {
        let mut pre_tasks = self.get_query_selector(selector, None);
        let get_box_model = dom_tasks::GetBoxModelTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id.map(Into::into)))
            .selector(selector.to_owned())
            .build()
            .expect("build GetBoxModelTaskBuilder should success.");
        pre_tasks.push(get_box_model.into());
        pre_tasks
    }

    pub fn get_box_model_by_selector(
        &mut self,
        selector: &str
    ) {
        let tasks = self.get_box_model_by_selector_task_impl(selector, None);
        self.execute_tasks(tasks);
    }

    pub fn get_box_model_by_selector_named(&mut self, selector: &str, name: &str) {
        let tasks = self.get_box_model_by_selector_task_impl(selector, Some(name.into()));
        self.execute_tasks(tasks);
    }

    fn get_document_box_model_impl(&mut self, name: Option<&str>) -> Vec<TaskDescribe> {
        let get_document_task = self.get_document_task(Some(1));
        let task = dom_tasks::GetBoxModelTaskBuilder::default()
            .common_fields(self.get_common_field(name.map(Into::into)))
            .build()
            .expect("GetBoxModelTaskBuilder should success.");
        vec![get_document_task, task.into()]
    }
}