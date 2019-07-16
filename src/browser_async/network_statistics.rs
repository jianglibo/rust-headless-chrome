use super::task_describe::network_events;

#[derive(Debug)]
pub struct NetworkStatistics {
    request_will_be_sent_events: Vec<network_events::RequestWillBeSent>,
    loading_failed_events: Vec<network_events::LoadingFailed>,
}

impl std::default::Default for NetworkStatistics {
    fn default() -> Self {
        Self {
            request_will_be_sent_events: Vec::new(),
            loading_failed_events: Vec::new(),
        }
    }
}

impl NetworkStatistics {
    pub fn request_will_be_sent(&mut self, event: network_events::RequestWillBeSent) {
        self.request_will_be_sent_events.push(event);
    }

    pub fn loading_failed(&mut self, event: network_events::LoadingFailed) {
        self.loading_failed_events.push(event);
    }

    pub fn find_request_will_send(
        &mut self,
        request_id: &str,
    ) -> &network_events::RequestWillBeSent {
        self.request_will_be_sent_events.iter().rev().find(|rs|rs.get_request_id_ref() == request_id)
            .expect("cannot find the request by request_id!")
    }

    pub fn list_request_urls(&self) -> Vec<&str> {
        self.request_will_be_sent_events.iter().map(|e|&*e.get_request_object().url).collect()
    }

    pub fn list_request_urls_end_with(&self, ext: &str) -> Vec<&str> {
        self.request_will_be_sent_events.iter().map(|e|&*e.get_request_object().url).filter(|url|url.ends_with(ext)).collect()
    }

    pub fn list_request_urls_contains(&self, ext: &str) -> Vec<&str> {
        self.request_will_be_sent_events.iter().map(|e|&*e.get_request_object().url).filter(|url|url.contains(ext)).collect()
    }
}