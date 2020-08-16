use crate::messages::AllCommands;

/// Result of a single step of the loop
#[derive(Debug)]
pub enum StepResult<'a> {
    /// A status was produced
    Status(Status<'a>),
    /// Nothing was produced, try again
    Nothing,
}

/// Status produced by the loop
#[derive(Debug)]
pub enum Status<'a> {
    /// A message was produced
    Message(AllCommands<'a>),
    /// The user quit the loop
    Quit,
    /// Loop run to completion
    Eof,
}
