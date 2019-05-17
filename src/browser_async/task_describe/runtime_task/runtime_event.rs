use crate::protocol::{page, network};

#[derive(Debug)]
pub struct ConsoleAPICalled {}

#[derive(Debug)]
pub struct ExceptionRevoked {}

#[derive(Debug)]
pub struct ExceptionThrown {}

#[derive(Debug)]
pub struct ExecutionContextCreated {}

#[derive(Debug)]
pub struct ExecutionContextDestroyed {}

#[derive(Debug)]
pub struct ExecutionContextsCleared {}

#[derive(Debug)]
pub struct InspectRequested {}