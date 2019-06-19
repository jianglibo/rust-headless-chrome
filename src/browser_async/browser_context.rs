use super::Tab;
use std::collections::HashMap;
use std::cmp::Ordering;


pub struct BrowserContexts<'a> {
    pub all_tabs: &'a mut [Tab],
}

impl<'a> BrowserContexts<'a> {
    pub fn get_browser_contexts(&mut self) -> Vec<BrowserContext> {
        let mut ctx_map: HashMap<String, Vec<&mut Tab>> = HashMap::new();
        for tab in self.all_tabs.iter_mut() {
            if let Some(browser_context_id) = tab.target_info.browser_context_id.clone() {
                ctx_map.entry(browser_context_id).or_insert(Vec::new()).push(tab);
            }
        }
        ctx_map.into_iter().map(|(_, tabs)|BrowserContext{tabs: tabs}).collect()
    }

    pub fn deduplicate(&mut self) {
        for mut bc in self.get_browser_contexts() {
            bc.deduplicate();
        }
    }
}

pub struct BrowserContext<'a> {
    pub tabs: Vec<&'a mut Tab>,
}

impl<'a> BrowserContext<'a> {
    pub fn get_last_opened_tab(&mut self) -> Option<&mut Tab> {
        if let Some((first, rest)) = self.tabs.split_first_mut() {
            let mut last_opened = first;
            for tb in rest {
                if tb.created_at > last_opened.created_at {
                    last_opened = tb;
                }
            }
            Some(last_opened)
        } else {
            None
        }
    }

    pub fn activate_last_opened_tab(&mut self) {
        if let Some(tab) = self.get_last_opened_tab() {
            tab.bring_to_front();
        }
    }

    fn sort_tabs(&mut self) {
       self.tabs.sort_unstable_by(|a, b| {
           let url_ordering = a.get_url().cmp(b.get_url());
           if let Ordering::Equal = url_ordering {
               a.created_at.cmp(&b.created_at)
           } else {
               url_ordering
           }
       }); 
    }

    pub fn deduplicate(&mut self) {
        self.sort_tabs();
        let _ = self.tabs.iter_mut().scan(None, |state: &mut Option<String>, cur_tab| {
            if state.as_ref().map(String::as_str) == Some(cur_tab.get_url()) {
                cur_tab.page_close();
            } else {
                state.replace(cur_tab.get_url().to_string());
            }
            Some(1)
        }).count();
    }
}