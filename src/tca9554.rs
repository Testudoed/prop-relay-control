/// TCA9554 I2C I/O Expander driver for 8-channel relay control
/// Address: 0x20
use embedded_hal_async::i2c::I2c;

pub const TCA9554_ADDRESS: u8 = 0x20;

#[repr(u8)]
enum Register {
    InputPort = 0x00,
    OutputPort = 0x01,
    Polarity = 0x02,
    Configuration = 0x03,
}

pub struct Tca9554<I2C> {
    i2c: I2C,
    address: u8,
    output_state: u8,
}

impl<I2C, E> Tca9554<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            output_state: 0x00,
        }
    }

    pub async fn init(&mut self) -> Result<(), E> {
        // Configure all pins as outputs
        self.write_register(Register::Configuration, 0x00).await?;
        // Set all outputs low
        self.write_register(Register::OutputPort, 0x00).await?;
        self.output_state = 0x00;
        Ok(())
    }

    async fn write_register(&mut self, register: Register, value: u8) -> Result<(), E> {
        self.i2c.write(self.address, &[register as u8, value]).await
    }

    pub async fn set_pin_high(&mut self, pin: u8) -> Result<(), E> {
        if pin >= 8 {
            return Ok(());
        }
        self.output_state |= 1 << pin;
        self.write_register(Register::OutputPort, self.output_state)
            .await
    }

    pub async fn set_pin_low(&mut self, pin: u8) -> Result<(), E> {
        if pin >= 8 {
            return Ok(());
        }
        self.output_state &= !(1 << pin);
        self.write_register(Register::OutputPort, self.output_state)
            .await
    }

    pub async fn all_off(&mut self) -> Result<(), E> {
        self.output_state = 0x00;
        self.write_register(Register::OutputPort, 0x00).await
    }

    pub fn get_output_state(&self) -> u8 {
        self.output_state
    }
}
