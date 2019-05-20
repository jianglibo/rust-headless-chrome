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
    ($target_type:ty) => {
        impl HasCommonField for $target_type {
            fn get_common_fields(&self) -> &CommonDescribeFields {
                &self.common_fields
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::*;
    use crate::browser_async::task_describe::{TaskDescribe, PageEvent};
    // use crate::browser_async::task_describe::page_events::PageCreated;


    #[test]
    fn test_impl_into_task_describe_macro() {
        // let () = PageEvent::PageCreated;
        // impl_into_task_describe!(TaskDescribe::PageEvent, PageEvent::PageCreated, PageCreated);
        assert!(true);
    }
}
