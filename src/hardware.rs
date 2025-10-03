/// Hardware abstraction for ESP32-S3-ETH-8DI-8RO module
///
/// Pin mapping from interface_description.md:
/// - Digital Inputs: GPIO4-11 (IN1-IN8)
/// - Relays: I2C I/O Expander (TCA9554) on GPIO41/42 (SCL/SDA)
/// - W5500 Ethernet: SPI on GPIO12-16, GPIO39
/// - Buzzer: GPIO46

/// Pin number constants
pub mod pins {
    // Digital Inputs
    pub const DI1: u8 = 4;
    pub const DI2: u8 = 5;
    pub const DI3: u8 = 6;
    pub const DI4: u8 = 7;
    pub const DI5: u8 = 8;
    pub const DI6: u8 = 9;
    pub const DI7: u8 = 10;
    pub const DI8: u8 = 11;

    // I2C for relay expander
    pub const I2C_SCL: u8 = 41;
    pub const I2C_SDA: u8 = 42;

    // W5500 Ethernet
    pub const ETH_MOSI: u8 = 13;
    pub const ETH_MISO: u8 = 14;
    pub const ETH_SCLK: u8 = 15;
    pub const ETH_CS: u8 = 16;
    pub const ETH_INT: u8 = 12;
    pub const ETH_RST: u8 = 39;

    pub const BUZZER: u8 = 46;
}

/// Digital input identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
#[repr(u8)]
pub enum DigitalInput {
    DI1 = 0,
    DI2 = 1,
    DI3 = 2,
    DI4 = 3,
    DI5 = 4,
    DI6 = 5,
    DI7 = 6,
    DI8 = 7,
}

/// Relay output identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
#[repr(u8)]
pub enum RelayOutput {
    Relay1 = 0,
    Relay2 = 1,
    Relay3 = 2,
    Relay4 = 3,
    Relay5 = 4,
    Relay6 = 5,
    Relay7 = 6,
    Relay8 = 7,
}

/// Relay state
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum RelayState {
    High,
    Low,
}
