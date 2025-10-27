use embassy_time::{Duration, Instant};

use crate::hardware::{DigitalInput, RelayOutput, RelayState};

/// Single step in a relay sequence
#[derive(Debug, Clone, Copy)]
pub struct SequenceStep {
    pub relay: RelayOutput,
    pub state: RelayState,
    pub duration_ms: u32,
}

impl SequenceStep {
    pub const fn new(relay: RelayOutput, state: RelayState, duration_ms: u32) -> Self {
        Self {
            relay,
            state,
            duration_ms,
        }
    }
}

/// Configuration for a trigger-to-sequence mapping
#[derive(Debug, Clone, Copy)]
pub struct SequenceConfig {
    /// Which digital input triggers this sequence
    pub trigger: DigitalInput,
    /// Cooldown duration in milliseconds
    pub cooldown_ms: u32,
    /// The sequence steps to execute
    pub sequence: &'static [SequenceStep],
    /// Optional name for logging
    pub name: &'static str,
}

impl SequenceConfig {
    pub const fn new(
        trigger: DigitalInput,
        cooldown_ms: u32,
        sequence: &'static [SequenceStep],
        name: &'static str,
    ) -> Self {
        Self {
            trigger,
            cooldown_ms,
            sequence,
            name,
        }
    }
}

/// Manages sequence dispatch and per-sequence cooldown tracking
pub struct SequenceDispatcher {
    cooldowns: [Option<Instant>; 8],
    cooldown_durations: [Duration; 8],
}

impl SequenceDispatcher {
    /// Create a new dispatcher from a sequence configuration array
    pub fn new(configs: &[SequenceConfig]) -> Self {
        let mut cooldown_durations = [Duration::from_millis(0); 8];

        // Set cooldown duration for each configured input
        for config in configs {
            let idx = config.trigger as usize;
            cooldown_durations[idx] = Duration::from_millis(config.cooldown_ms as u64);
        }

        Self {
            cooldowns: [None; 8],
            cooldown_durations,
        }
    }

    /// Check if an input is currently in cooldown
    pub fn is_cooling_down(&self, input: DigitalInput) -> bool {
        let idx = input as usize;
        if let Some(last) = self.cooldowns[idx] {
            Instant::now().duration_since(last) < self.cooldown_durations[idx]
        } else {
            false
        }
    }

    /// Mark an input as triggered (starts cooldown)
    pub fn mark_triggered(&mut self, input: DigitalInput) {
        self.cooldowns[input as usize] = Some(Instant::now());
    }

    /// Get remaining cooldown time in milliseconds
    pub fn remaining_ms(&self, input: DigitalInput) -> u64 {
        let idx = input as usize;
        if let Some(last) = self.cooldowns[idx] {
            let elapsed = Instant::now().duration_since(last);
            if elapsed < self.cooldown_durations[idx] {
                (self.cooldown_durations[idx] - elapsed).as_millis()
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Find a matching sequence configuration for the given input
    pub fn find_config<'a>(
        &self,
        configs: &'a [SequenceConfig],
        input: DigitalInput,
    ) -> Option<&'a SequenceConfig> {
        configs.iter().find(|cfg| cfg.trigger == input)
    }
}

// Pre-defined sequences
pub const JUMP_SCARE: &[SequenceStep] = &[
    SequenceStep::new(RelayOutput::Relay1, RelayState::High, 1000),
    SequenceStep::new(RelayOutput::Relay1, RelayState::Low, 0),
];

pub const SNAKE_SEQUENCE: &[SequenceStep] = &[
    SequenceStep::new(RelayOutput::Relay2, RelayState::High, 100),
    SequenceStep::new(RelayOutput::Relay2, RelayState::Low, 900),
    SequenceStep::new(RelayOutput::Relay2, RelayState::High, 200),
    SequenceStep::new(RelayOutput::Relay2, RelayState::Low, 800),
    SequenceStep::new(RelayOutput::Relay2, RelayState::High, 200),
    SequenceStep::new(RelayOutput::Relay2, RelayState::Low, 800),
    SequenceStep::new(RelayOutput::Relay2, RelayState::High, 250),
    SequenceStep::new(RelayOutput::Relay2, RelayState::Low, 250),
    SequenceStep::new(RelayOutput::Relay2, RelayState::High, 250),
    SequenceStep::new(RelayOutput::Relay2, RelayState::Low, 250),
    SequenceStep::new(RelayOutput::Relay2, RelayState::High, 500),
    SequenceStep::new(RelayOutput::Relay2, RelayState::Low, 0),
];
