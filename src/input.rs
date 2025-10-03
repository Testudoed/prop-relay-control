/// Digital input monitoring with debouncing and cooldown
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Timer};
use esp_hal::gpio::Input;

use crate::hardware::DigitalInput;

/// Input trigger event
#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    pub input: DigitalInput,
    pub timestamp_ms: u64,
}

/// Channel for input events (queue size: 16)
pub type InputEventChannel = Channel<CriticalSectionRawMutex, InputEvent, 16>;

/// Monitor a digital input with interrupt-based detection and debouncing
pub async fn input_monitor_task<const PIN: u8>(
    mut pin: Input<'static>,
    input_id: DigitalInput,
    debounce_ms: u32,
    channel: &'static InputEventChannel,
) -> ! {
    let debounce_duration = Duration::from_millis(debounce_ms as u64);
    let mut last_trigger = Instant::MIN;

    defmt::info!("Input monitor started: {:?} (GPIO{})", input_id, PIN);

    loop {
        // Wait for rising edge (sensor activation)
        pin.wait_for_rising_edge().await;

        let now = Instant::now();

        // Debounce check
        if now.duration_since(last_trigger) >= debounce_duration {
            last_trigger = now;
            let timestamp_ms = now.as_millis();

            let event = InputEvent {
                input: input_id,
                timestamp_ms,
            };

            if channel.try_send(event).is_err() {
                defmt::warn!("Event channel full, dropping {:?}", input_id);
            } else {
                defmt::info!("Input triggered: {:?}", input_id);
            }

            Timer::after(debounce_duration).await;
        }
    }
}

/// Per-input cooldown tracker
pub struct CooldownTracker {
    cooldowns: [Option<Instant>; 8],
    cooldown_duration: Duration,
}

impl CooldownTracker {
    pub fn new(cooldown_ms: u32) -> Self {
        Self {
            cooldowns: [None; 8],
            cooldown_duration: Duration::from_millis(cooldown_ms as u64),
        }
    }

    pub fn is_cooling_down(&self, input: DigitalInput) -> bool {
        if let Some(last) = self.cooldowns[input as usize] {
            Instant::now().duration_since(last) < self.cooldown_duration
        } else {
            false
        }
    }

    pub fn mark_triggered(&mut self, input: DigitalInput) {
        self.cooldowns[input as usize] = Some(Instant::now());
    }

    pub fn remaining_ms(&self, input: DigitalInput) -> u64 {
        if let Some(last) = self.cooldowns[input as usize] {
            let elapsed = Instant::now().duration_since(last);
            if elapsed < self.cooldown_duration {
                (self.cooldown_duration - elapsed).as_millis()
            } else {
                0
            }
        } else {
            0
        }
    }
}
