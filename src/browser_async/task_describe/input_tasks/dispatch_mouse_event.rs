use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTask, AsMethodCallString, HasCommonField, CanCreateMethodString,};
use crate::protocol::{input, network};
use failure;

#[derive(Debug, Clone)]
pub enum MouseButton {
    NoButton,
    Left,
    Middle,
    Right,
    Back,
    Forward,
}

impl From<MouseButton> for &'static str {
    fn from(mb: MouseButton) -> &'static str {
        match mb {
            MouseButton::NoButton => "none",
            MouseButton::Left => "left",
            MouseButton::Middle => "middle",
            MouseButton::Right => "right",
            MouseButton::Back => "back",
            MouseButton::Forward => "forward",
        }
    }
}

#[derive(Debug, Clone)]
pub enum MouseEventType {
    Pressed,
    Released,
    Moved,
    Wheel,
}

impl Default for MouseEventType {
    fn default() -> Self {
        MouseEventType::Moved
    }
}

impl From<MouseEventType> for &'static str {
    fn from(met: MouseEventType) -> &'static str {
        match met {
            MouseEventType::Pressed => "mousePressed",
            MouseEventType::Released => "mouseReleased",
            MouseEventType::Moved => "mouseMoved",
            MouseEventType::Wheel => "mouseWheel",
        }
    }
}

#[derive(Debug, Clone)]
pub enum PointerType {
    Mouse,
    Pen,
}

impl From<PointerType> for &'static str {
    fn from(pt: PointerType) -> &'static str {
        match pt {
            PointerType::Mouse => "mouse",
            PointerType::Pen => "pen",
        }
    }
}

#[derive(Debug, Builder, Default)]
#[builder(setter(into))]
pub struct DispatchMouseEventTask {
    pub common_fields: CommonDescribeFields,
    pub event_type: MouseEventType,
    pub x: Option<f64>,
    pub y: Option<f64>,
    #[builder(default = "None")]
    pub modifiers: Option<u8>,
    #[builder(default = "None")]
    pub timestamp: Option<network::TimeSinceEpoch>,
    #[builder(default = "None")]
    pub button: Option<MouseButton>,
    #[builder(default = "None")]
    pub buttons: Option<u8>,
    #[builder(default = "None")]
    pub click_count: Option<u8>,
    #[builder(default = "None")]
    pub delta_x: Option<f64>,
    #[builder(default = "None")]
    pub delta_y: Option<f64>,
    #[builder(default = "None")]
    pub pointer_type: Option<PointerType>,
}

impl_has_common_fields!(DispatchMouseEventTask);

impl AsMethodCallString for DispatchMouseEventTask {
    fn get_method_str(&self) -> Result<String, failure::Error> {
        let method = input::methods::DispatchMouseEvent {
            event_type: self.event_type.clone().into(),
            x: self.x.expect("mouse point x should be assigned."),
            y: self.y.expect("mouse point y should be assigned."),
            button: self.button.as_ref().cloned().map(Into::into),
            click_count: self.click_count,
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::DispatchMouseEvent, DispatchMouseEventTask);