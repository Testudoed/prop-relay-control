/// Relay sequence execution using TCA9554 I2C expander
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use embedded_hal_async::i2c::I2c;

use crate::hardware::{RelayOutput, RelayState};
use crate::sequence::SequenceStep;
use crate::tca9554::Tca9554;

/// Relay controller managing 8 relay outputs via I2C
pub struct RelayController<I2C> {
    expander: Mutex<CriticalSectionRawMutex, Tca9554<I2C>>,
}

impl<I2C, E> RelayController<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(expander: Tca9554<I2C>) -> Self {
        Self {
            expander: Mutex::new(expander),
        }
    }

    pub async fn init(&self) -> Result<(), E> {
        let mut expander = self.expander.lock().await;
        expander.init().await?;
        defmt::info!("Relay controller initialized - all relays OFF");
        Ok(())
    }

    pub async fn set_relay(&self, relay: RelayOutput, state: RelayState) -> Result<(), E> {
        let mut expander = self.expander.lock().await;
        let pin = relay as u8;
        match state {
            RelayState::High => expander.set_pin_high(pin).await?,
            RelayState::Low => expander.set_pin_low(pin).await?,
        }
        defmt::debug!("Relay {} -> {:?}", pin + 1, state);
        Ok(())
    }

    pub async fn execute_sequence(&self, sequence: &[SequenceStep]) -> Result<(), E> {
        defmt::info!("Executing sequence ({} steps)", sequence.len());

        for step in sequence {
            defmt::debug!(
                "  Step: {:?} -> {:?} for {}ms",
                step.relay,
                step.state,
                step.duration_ms
            );

            self.set_relay(step.relay, step.state).await?;
            Timer::after(Duration::from_millis(step.duration_ms as u64)).await;
        }

        defmt::info!("Sequence complete");
        Ok(())
    }

    pub async fn all_off(&self) -> Result<(), E> {
        defmt::info!("Turning all relays OFF");
        let mut expander = self.expander.lock().await;
        expander.all_off().await
    }
}
