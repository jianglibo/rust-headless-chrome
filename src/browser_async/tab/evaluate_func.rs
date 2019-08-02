use super::super::super::browser_async::{create_unique_prefixed_id, TaskId};
use super::super::task_describe::{TaskDescribe, runtime_tasks};
use super::Tab;
use super::super::super::protocol::{runtime};
use log::*;

impl Tab {
    pub fn evaluate_expression<T: AsRef<str>>(&mut self, expression: T) {
        let task = self.evaluate_expression_task(expression);
        self.execute_one_task(task);
    }

    pub fn evaluate_expression_prefixed<T: AsRef<str>>(&mut self, expression: T, prefix: &str) {
        let name = create_unique_prefixed_id(prefix);
        self.evaluate_expression_named(expression, name.as_str());
    }

    pub fn evaluate_expression_named<T: AsRef<str>>(&mut self, expression: T, name: &str) {
        let task = self.evaluate_expression_task_named(expression, name);
        self.execute_one_task(task);
    }

    pub fn evaluate_expression_task_named<T: AsRef<str>>(
        &mut self,
        expression: T,
        task_id: &str,
    ) -> TaskDescribe {
        self.evaluate_expression_task_impl(expression, Some(task_id))
    }

    pub fn evaluate_expression_task_prefixed(
        &mut self,
        expression: &str,
        prefix: &str,
    ) -> TaskDescribe {
        let name = create_unique_prefixed_id(prefix);
        self.evaluate_expression_task_named(expression, name.as_str())
    }

    pub fn evaluate_expression_task<T: AsRef<str>>(&self, expression: T) -> TaskDescribe {
        self.evaluate_expression_task_impl(expression, None)
    }

    fn evaluate_expression_task_impl<T: AsRef<str>>(
        &self,
        expression: T,
        manual_task_id: Option<&str>,
    ) -> TaskDescribe {
        runtime_tasks::EvaluateTaskBuilder::default()
            .expression(expression.as_ref().to_string())
            .common_fields(self.get_common_field(manual_task_id.map(Into::into)))
            .build()
            .expect("build EvaluateTaskBuilder should success.")
            .into()
    }

    pub fn evaluate_task(
        &self,
        evaluate_task_builder: runtime_tasks::EvaluateTaskBuilder,
    ) -> TaskDescribe {
        self.evaluate_task_impl(evaluate_task_builder, None)
    }

    pub fn evaluate_task_named(
        &self,
        evaluate_task_builder: runtime_tasks::EvaluateTaskBuilder,
        name: &str,
    ) -> TaskDescribe {
        self.evaluate_task_impl(evaluate_task_builder, Some(name))
    }

    fn evaluate_task_impl(
        &self,
        mut evaluate_task_builder: runtime_tasks::EvaluateTaskBuilder,
        manual_task_id: Option<&str>,
    ) -> TaskDescribe {
        let task = evaluate_task_builder
            .common_fields(self.get_common_field(manual_task_id.map(Into::into)))
            .build();
        match task {
            Ok(task) => task.into(),
            Err(err) => {
                error!("build evaluate task error: {:?}", err);
                panic!("build evaluate task error: {:?}", err);
            }
        }
    }

    pub fn evaluate(&mut self, evaluate_task_builder: runtime_tasks::EvaluateTaskBuilder) {
        self.evaluate_impl(evaluate_task_builder, None)
    }

    pub fn evaluate_named(
        &mut self,
        evaluate_task_builder: runtime_tasks::EvaluateTaskBuilder,
        name: &str,
    ) {
        self.evaluate_impl(evaluate_task_builder, Some(name))
    }

    fn evaluate_impl(
        &mut self,
        evaluate_task_builder: runtime_tasks::EvaluateTaskBuilder,
        manual_task_id: Option<&str>,
    ) {
        let task = self.evaluate_task_impl(evaluate_task_builder, manual_task_id);
        self.execute_one_task(task);
    }

    fn get_properties_by_object_id_impl(
        &mut self,
        object_id: runtime::RemoteObjectId,
        name: Option<&str>,
    ) {
        let task = runtime_tasks::GetPropertiesTaskBuilder::default()
            .object_id(object_id)
            .common_fields(self.get_common_field(name.map(Into::into)))
            .build()
            .expect("build GetPropertiesTaskBuilder should success.");
        self.execute_one_task(task.into());
    }

    pub fn get_properties_by_object_id(&mut self, object_id: runtime::RemoteObjectId) {
        self.get_properties_by_object_id_impl(object_id, None);
    }

    pub fn get_properties_by_object_id_named(&mut self, object_id: runtime::RemoteObjectId, name: &str) {
        self.get_properties_by_object_id_impl(object_id, Some(name));
    }

    pub fn get_properties(
        &mut self,
        mut get_properties_task_builder: runtime_tasks::GetPropertiesTaskBuilder,
        manual_task_id: Option<TaskId>,
    ) {
        let task = get_properties_task_builder
            .common_fields(self.get_common_field(manual_task_id))
            .build()
            .expect("GetBoxModelTaskBuilder should success.");
        self.execute_one_task(task.into());
    }

    fn evaluate_expression_and_get_properties_task_impl(&self, expression: &str, name: Option<&str>) -> Vec<TaskDescribe> {
        let evaluate_task = self.evaluate_expression_task(expression);
        let get_properties_task =  runtime_tasks::GetPropertiesTaskBuilder::default()
            .common_fields(self.get_common_field(name.map(Into::into)))
            .build().expect("GetBoxModelTaskBuilder should build success.");
        vec![evaluate_task, get_properties_task.into()]
    }

    /// Please listen on MethodCallDone::GetProperties(task).
    pub fn evaluate_expression_and_get_properties_named(&mut self, expression: &str, name: &str) {
        let tasks = self.evaluate_expression_and_get_properties_task_impl(expression, Some(name));
        self.execute_tasks(tasks);
    }
}