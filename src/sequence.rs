use crate::hardware::{RelayOutput, RelayState};
use crate::relay::SequenceStep;

pub const JUMP_SCARE: &[SequenceStep] = &[
    SequenceStep::new(RelayOutput::Relay1, RelayState::High, 1000),
    SequenceStep::new(RelayOutput::Relay1, RelayState::Low, 0),
];

pub const SNAKE_SEQUENCE: &[SequenceStep] = &[
    SequenceStep::new(RelayOutput::Relay2, RelayState::High, 500),
    SequenceStep::new(RelayOutput::Relay2, RelayState::Low, 500),
    SequenceStep::new(RelayOutput::Relay2, RelayState::High, 500),
    SequenceStep::new(RelayOutput::Relay2, RelayState::Low, 500),
    SequenceStep::new(RelayOutput::Relay2, RelayState::High, 500),
    SequenceStep::new(RelayOutput::Relay2, RelayState::Low, 500),
    SequenceStep::new(RelayOutput::Relay2, RelayState::High, 500),
    SequenceStep::new(RelayOutput::Relay2, RelayState::Low, 500),
];
