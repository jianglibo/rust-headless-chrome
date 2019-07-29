// https://doc.rust-lang.org/reference/macros-by-example.html
// https://hub.packtpub.com/creating-macros-in-rust-tutorial

#[macro_export]
macro_rules! impl_into_task_describe {
    // ( $i:item ) => (println!("item: {}", stringify!($i)))
    // ( $i:stmt ) => (println!("item: {}", stringify!($i));) // ok
    // ( $i:path ) => (println!("item: {}", stringify!($i));) // ok
    // ( $i:path,$j:literal ) => (println!("item: {} {}", stringify!($i), $j);) // ok
    ($evt_name:path, $evt_variant:path, $variant_value:path) => {
        impl std::convert::From<$variant_value> for TaskDescribe {
            fn from(event: $variant_value) -> Self {
                let pe = $evt_variant(event);
                $evt_name(pe)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_new_for_event {
    ($target_event:path, $raw_event:path) => {
        impl $target_event {
            pub fn new(raw_event: $raw_event) -> Self {
                Self{raw_event}
            }
        }
    };
}

#[macro_export]
macro_rules! wrapper_raw_event {
    ($top_variant:path, $sub_variant:path, $struct_name:ident, $raw_event:ty) => {
        #[derive(Debug)]
        pub struct $struct_name {
            raw_event: $raw_event,
        }

        impl $struct_name {
            pub fn new(raw_event: $raw_event) -> Self {
                Self{raw_event}
            }
        }
        impl std::convert::From<$struct_name> for TaskDescribe {
            fn from(event: $struct_name) -> Self {
                let pe = $sub_variant(event);
                $top_variant(pe)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_has_common_fields {
    ($target_type:ty, $task_name:literal) => {
        impl HasCommonField for $target_type {
            const TASK_NAME: &'static str = $task_name;
            fn get_common_fields(&self) -> &CommonDescribeFields {
                &self.common_fields
            }
            fn get_common_fields_mut(&mut self) -> &mut CommonDescribeFields {
                &mut self.common_fields
            }
        }
    };
}

#[macro_export]
macro_rules! impl_has_task_name_for_task_describe {
    ([$($target_task:path),*], [$($browser_task:path),*]) => {
        impl HasTaskName for TaskDescribe {
            fn get_task_name(&self) -> &str {
                 match self {
                    TaskDescribe::TargetCallMethod(target_call) => 
                        match target_call {
                            $($target_task(task) => task.get_task_name(),)*
                        }
                    TaskDescribe::BrowserCallMethod(browser_call) => 
                        match browser_call {
                            $($browser_task(task) => task.get_task_name(),)*
                        }
                    _ => ""
                 }
            }
        }
    };
}

// impl HasCommonField for TaskDescribe {
//     fn get_task_name(&self) -> &str {
//          match self {
//             TaskDescribe::TargetCallMethod(target_call) => 
//                 match target_call {
//                 TargetCallMethodTask::QuerySelector(task) => task.get_task_name(),
//                 TargetCallMethodTask::DescribeNode(task) => task.get_task_name(),
//                 TargetCallMethodTask::PrintToPDF(task) => task.get_task_name(),
//                 TargetCallMethodTask::GetBoxModel(task) => task.get_task_name(),
//                 TargetCallMethodTask::GetContentQuads(task) => task.get_task_name(),
//                 TargetCallMethodTask::CaptureScreenshot(task) => task.get_task_name(),
//                 TargetCallMethodTask::GetDocument(task) => task.get_task_name(),
//                 TargetCallMethodTask::NavigateTo(task) => task.get_task_name(),
//                 TargetCallMethodTask::PageEnable(task) => task.get_task_name(),
//                 TargetCallMethodTask::RuntimeEnable(task) => task.get_task_name(),
//                 TargetCallMethodTask::Evaluate(task) => task.get_task_name(),
//                 TargetCallMethodTask::GetProperties(task) => task.get_task_name(),
//                 TargetCallMethodTask::RuntimeCallFunctionOn(task) => task.get_task_name(),
//                 TargetCallMethodTask::NetworkEnable(task) => task.get_task_name(),
//                 TargetCallMethodTask::SetRequestInterception(task) => task.get_task_name(),
//                 TargetCallMethodTask::GetResponseBodyForInterception(task) => task.get_task_name(),
//                 TargetCallMethodTask::ContinueInterceptedRequest(task) => task.get_task_name(),
//                 TargetCallMethodTask::PageReload(task) => task.get_task_name(),
//                 TargetCallMethodTask::GetLayoutMetrics(task) => task.get_task_name(),
//                 TargetCallMethodTask::BringToFront(task) => task.get_task_name(),
//                 TargetCallMethodTask::PageClose(task) => task.get_task_name(),
//                 TargetCallMethodTask::DispatchMouseEvent(task) => task.get_task_name(),
//                 TargetCallMethodTask::CanEmulate(task) => task.get_task_name(),
//                 TargetCallMethodTask::SetDeviceMetricsOverride(task) => task.get_task_name(),
//                 TargetCallMethodTask::SetLifecycleEventsEnabled(task) => task.get_task_name(),
//             }
//             TaskDescribe::BrowserCallMethod(browser_call) => match browser_call {
//                 BrowserCallMethodTask::CreateTarget(task) => task.get_task_name(),
//                 BrowserCallMethodTask::SetDiscoverTargets(task) => task.get_task_name(),
//                 BrowserCallMethodTask::SetIgnoreCertificateErrors(task) => task.get_task_name(),
//                 BrowserCallMethodTask::SecurityEnable(task) => task.get_task_name(),
//                 BrowserCallMethodTask::AttachedToTarget(task) => task.get_task_name(),
//                 BrowserCallMethodTask::CloseTarget(task) => task.get_task_name(),
//                 BrowserCallMethodTask::ActivateTarget(task) => task.get_task_name(),
//             }
//             _ => {
//                 ""
//             }   
//          }
// }
// }
