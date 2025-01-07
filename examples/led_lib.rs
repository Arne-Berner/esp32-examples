use esp_idf_svc::hal::gpio::{Output, OutputPin, Pin, PinDriver};

pub struct LED<'a, S: Pin> {
    led_driver: PinDriver<'a, S, Output>,
}

impl<S> LED<'_, S>
where
    S: OutputPin,
{
    pub fn new(led: S) -> anyhow::Result<Self, anyhow::Error> {
        let led_driver = PinDriver::output(led)?;
        Ok(Self { led_driver })
    }

    // Get this out of here
    pub fn set_led(self: &mut Self, high: bool) -> anyhow::Result<(), anyhow::Error> {
        if high {
            self.led_driver.set_high()?;
        } else {
            self.led_driver.set_low()?;
        }
        Ok(())
    }
}
