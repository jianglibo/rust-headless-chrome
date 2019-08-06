use super::super::super::protocol::page;
use super::super::page_message::ChangingFrame;
use log::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ChangingFrames {
    pub changing_frames: HashMap<page::FrameId, ChangingFrame>,
}

impl ChangingFrames {
    pub fn _frame_started_loading(&mut self, frame_id: String) {
        if let Some(changing_frame) = self.changing_frames.get_mut(&frame_id) {
            *changing_frame = ChangingFrame::StartedLoading(frame_id);
        } else {
            self.changing_frames
                .insert(frame_id.clone(), ChangingFrame::StartedLoading(frame_id));
        }
    }

    pub fn len(&self) -> usize {
        self.changing_frames.len()
    }

    pub fn count_stopped(&self) -> usize {
        self.changing_frames
            .values()
            .filter(|cm| match cm {
                ChangingFrame::StoppedLoading(_) | ChangingFrame::StopLoadingFrameId => true,
                _ => false,
            })
            .count()
    }

    pub fn _frame_stopped_loading<T: AsRef<str>>(&mut self, frame_id: T) {
        if let Some(changing_frame) = self.changing_frames.get_mut(frame_id.as_ref()) {
            if let ChangingFrame::Navigated(fm) = changing_frame {
                *changing_frame = ChangingFrame::StoppedLoading(fm.clone());
            } else {
                error!(
                    "get stopped, but no Navigated: -----------{:?}",
                    changing_frame
                );
            }
        } else {
            self.changing_frames.insert(
                frame_id.as_ref().to_string(),
                ChangingFrame::StopLoadingFrameId,
            );
            info!(
                "Cannot found frame with id when got _frame_stopped_loading: {:?}",
                frame_id.as_ref()
            );
        }
    }

    pub fn _frame_navigated(&mut self, frame: page::Frame) {
        if let Some(changing_frame) = self.changing_frames.get_mut(&frame.id) {
            *changing_frame = ChangingFrame::Navigated(frame);
        } else {
            info!(
                "Cannot found frame with id when got _frame_navigated, sometime chrome didn't emit other two events.: {:?}",
                frame,
            );
            self.changing_frames
                .insert(frame.id.clone(), ChangingFrame::Navigated(frame));
        }
    }

    pub fn _frame_attached(&mut self, frame_attached_params: page::events::FrameAttachedParams) {
        let frame_id = frame_attached_params.frame_id.clone();
        self.changing_frames.insert(
            frame_id.clone(),
            ChangingFrame::Attached(frame_attached_params),
        );
    }
    pub fn _frame_detached(&mut self, frame_id: &str) {
        self.changing_frames.remove(frame_id);
    }
    pub fn find_frame_by_id(&self, frame_id: &str) -> Option<&page::Frame> {
        match self.changing_frames.get(frame_id) {
            Some(ChangingFrame::Navigated(fm)) | Some(ChangingFrame::StoppedLoading(fm)) => {
                Some(fm)
            }
            _ => None,
        }
    }

    pub fn find_frame_by_name(&self, frame_name: &'static str) -> Option<&page::Frame> {
        self.changing_frames.values().find_map(|cf| match cf {
            ChangingFrame::Navigated(fr) | ChangingFrame::StoppedLoading(fr)
                if fr.name == Some(frame_name.into()) =>
            {
                Some(fr)
            }
            _ => None,
        })
    }

    pub fn main_frame(&self) -> Option<&page::Frame> {
        self.changing_frames.values().find_map(|cf| match cf {
            ChangingFrame::Navigated(fm) | ChangingFrame::StoppedLoading(fm)
                if fm.parent_id.is_none() =>
            {
                Some(fm)
            }
            _ => None,
        })
    }

    pub fn find_navigated_frame<F>(&self, mut filter: F) -> Option<&page::Frame>
    where
        F: FnMut(&page::Frame) -> bool,
    {
        self.changing_frames
            .values()
            .filter_map(|cf| match cf {
                ChangingFrame::Navigated(fm) | ChangingFrame::StoppedLoading(fm) => Some(fm),
                _ => None,
            })
            .find(|frame| filter(frame))
    }
}
