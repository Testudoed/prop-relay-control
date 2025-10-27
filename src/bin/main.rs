#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::timer::systimer::SystemTimer;
use panic_rtt_target as _;
use prop_relay_control::hardware::DigitalInput;
use prop_relay_control::input::{input_monitor_task, InputEventChannel};
use prop_relay_control::relay::RelayController;
use prop_relay_control::sequence::{
    SequenceConfig, SequenceDispatcher, JUMP_SCARE, SNAKE_SEQUENCE,
};
use prop_relay_control::tca9554::{Tca9554, TCA9554_ADDRESS};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

// Global input event channel
static INPUT_CHANNEL: InputEventChannel = embassy_sync::channel::Channel::new();

/// Sequence configuration registry
///
/// To add a new sequence:
/// 1. Define the sequence steps in src/sequence.rs (or use an existing one)
/// 2. Add a new SequenceConfig entry here with:
///    - Trigger input (which DI# activates it)
///    - Cooldown duration in milliseconds
///    - Reference to the sequence steps
///    - Name for logging
///
/// Example:
/// ```
/// const MY_SEQUENCE: &[SequenceStep] = &[
///     SequenceStep::new(RelayOutput::Relay3, RelayState::High, 2000),
///     SequenceStep::new(RelayOutput::Relay3, RelayState::Low, 0),
/// ];
///
/// // Then add to SEQUENCE_CONFIGS:
/// SequenceConfig::new(DigitalInput::DI3, 4000, MY_SEQUENCE, "My Effect"),
/// ```
const SEQUENCE_CONFIGS: &[SequenceConfig] = &[
    SequenceConfig::new(DigitalInput::DI1, 5000, JUMP_SCARE, "Jump Scare"),
    SequenceConfig::new(DigitalInput::DI2, 30000, SNAKE_SEQUENCE, "Snake Attack"),
    // Add more sequence mappings here...
];

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Prop Relay Controller starting...");

    // Initialize I2C for TCA9554 relay expander (async mode)
    let i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .expect("Failed to create I2C")
        .with_sda(peripherals.GPIO42)
        .with_scl(peripherals.GPIO41)
        .into_async();

    let tca9554 = Tca9554::new(i2c, TCA9554_ADDRESS);
    let relay_controller = RelayController::new(tca9554);

    if let Err(_) = relay_controller.init().await {
        defmt::error!("Failed to initialize relay controller");
    }

    // Initialize digital input pins (GPIO4-11)
    let input_cfg = InputConfig::default().with_pull(Pull::Up);
    let di1 = Input::new(peripherals.GPIO4, input_cfg.clone());
    let di2 = Input::new(peripherals.GPIO5, input_cfg.clone());
    let di3 = Input::new(peripherals.GPIO6, input_cfg.clone());
    let di4 = Input::new(peripherals.GPIO7, input_cfg.clone());
    let di5 = Input::new(peripherals.GPIO8, input_cfg.clone());
    let di6 = Input::new(peripherals.GPIO9, input_cfg.clone());
    let di7 = Input::new(peripherals.GPIO10, input_cfg.clone());
    let di8 = Input::new(peripherals.GPIO11, input_cfg.clone());

    info!("Hardware initialized, starting tasks...");

    // Spawn input monitor tasks for all 8 digital inputs
    spawner.spawn(di1_monitor_task(di1)).ok();
    spawner.spawn(di2_monitor_task(di2)).ok();
    spawner.spawn(di3_monitor_task(di3)).ok();
    spawner.spawn(di4_monitor_task(di4)).ok();
    spawner.spawn(di5_monitor_task(di5)).ok();
    spawner.spawn(di6_monitor_task(di6)).ok();
    spawner.spawn(di7_monitor_task(di7)).ok();
    spawner.spawn(di8_monitor_task(di8)).ok();

    // Spawn main control task
    spawner.spawn(control_task(relay_controller)).ok();

    info!("System ready - 8 input monitors active");
}

// Input monitor tasks
#[embassy_executor::task]
async fn di1_monitor_task(pin: Input<'static>) {
    input_monitor_task::<4>(pin, DigitalInput::DI1, 100, &INPUT_CHANNEL).await
}

#[embassy_executor::task]
async fn di2_monitor_task(pin: Input<'static>) {
    input_monitor_task::<5>(pin, DigitalInput::DI2, 100, &INPUT_CHANNEL).await
}

#[embassy_executor::task]
async fn di3_monitor_task(pin: Input<'static>) {
    input_monitor_task::<6>(pin, DigitalInput::DI3, 100, &INPUT_CHANNEL).await
}

#[embassy_executor::task]
async fn di4_monitor_task(pin: Input<'static>) {
    input_monitor_task::<7>(pin, DigitalInput::DI4, 100, &INPUT_CHANNEL).await
}

#[embassy_executor::task]
async fn di5_monitor_task(pin: Input<'static>) {
    input_monitor_task::<8>(pin, DigitalInput::DI5, 100, &INPUT_CHANNEL).await
}

#[embassy_executor::task]
async fn di6_monitor_task(pin: Input<'static>) {
    input_monitor_task::<9>(pin, DigitalInput::DI6, 100, &INPUT_CHANNEL).await
}

#[embassy_executor::task]
async fn di7_monitor_task(pin: Input<'static>) {
    input_monitor_task::<10>(pin, DigitalInput::DI7, 100, &INPUT_CHANNEL).await
}

#[embassy_executor::task]
async fn di8_monitor_task(pin: Input<'static>) {
    input_monitor_task::<11>(pin, DigitalInput::DI8, 100, &INPUT_CHANNEL).await
}

// Main control task
#[embassy_executor::task]
async fn control_task(relay_controller: RelayController<I2c<'static, esp_hal::Async>>) {
    info!("Control task started");
    info!(
        "Loaded {} sequence configuration(s)",
        SEQUENCE_CONFIGS.len()
    );

    // Create sequence dispatcher with our configurations
    let mut dispatcher = SequenceDispatcher::new(SEQUENCE_CONFIGS);

    loop {
        // Wait for input events
        let event = INPUT_CHANNEL.receive().await;
        info!("Received input event: {:?} triggered", event.input);

        // Check if this input is in cooldown
        if dispatcher.is_cooling_down(event.input) {
            let remaining = dispatcher.remaining_ms(event.input);
            info!(
                "Input {:?} cooling down, ignoring ({}ms remaining)",
                event.input, remaining
            );
            continue;
        }

        // Find matching sequence configuration
        if let Some(config) = dispatcher.find_config(SEQUENCE_CONFIGS, event.input) {
            info!("Executing sequence: {}", config.name);

            // Mark triggered (start cooldown)
            dispatcher.mark_triggered(event.input);

            // Execute sequence
            if let Err(_) = relay_controller.execute_sequence(config.sequence).await {
                defmt::error!("Failed to execute sequence: {}", config.name);
            }

            info!(
                "Sequence '{}' complete. Cooldown active for {}ms",
                config.name, config.cooldown_ms
            );
        } else {
            info!("No sequence mapped to {:?}", event.input);
        }
    }
}
