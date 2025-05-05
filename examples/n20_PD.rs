use encoder::Encoder;
use esp_idf_hal::delay::{self};
use esp_idf_hal::gpio::{Gpio10, Gpio7};
use esp_idf_hal::ledc::config::TimerConfig as LedcConfig;
use esp_idf_hal::ledc::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::sys::EspError;
use esp_idf_hal::task::*;
use esp_idf_hal::timer::*;
use esp_idf_svc::hal::gpio::PinDriver;
use l293x::L293x;
use std::cmp;

const K_P: f32 = 4.0;
const K_D: f32 = 0.0;
const ENCODERZIEL: i32 = 5647;

fn main() -> anyhow::Result<()> {
    esp_idf_hal::sys::link_patches();

    println!("Configuring output channel");

    let peripherals = Peripherals::take()?;

    let boot_pin = peripherals.pins.gpio0;
    let mut boot = PinDriver::input(boot_pin)?;
    boot.set_pull(esp_idf_hal::gpio::Pull::Up).unwrap();
    let gpio5_pin = peripherals.pins.gpio5;
    let mut gpio5 = PinDriver::input(gpio5_pin)?;
    gpio5.set_pull(esp_idf_hal::gpio::Pull::Up).unwrap();
    let led_pin = peripherals.pins.gpio17;
    let mut led_driver = PinDriver::output(led_pin)?;
    led_driver.set_high();

    // Motors
    // create driver
    let driver1 = LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &LedcConfig::new().frequency(5.kHz().into()),
    )?;
    let driver2 = LedcTimerDriver::new(
        peripherals.ledc.timer1,
        &LedcConfig::new().frequency(5.kHz().into()),
    )?;
    let driver3 = LedcTimerDriver::new(
        peripherals.ledc.timer2,
        &LedcConfig::new().frequency(5.kHz().into()),
    )?;
    let driver4 = LedcTimerDriver::new(
        peripherals.ledc.timer3,
        &LedcConfig::new().frequency(5.kHz().into()),
    )?;

    let m1_forward = LedcDriver::new(peripherals.ledc.channel0, driver1, peripherals.pins.gpio26)?;

    let m1_reverse = LedcDriver::new(peripherals.ledc.channel1, driver2, peripherals.pins.gpio21)?;

    let m2_forward = LedcDriver::new(peripherals.ledc.channel2, driver3, peripherals.pins.gpio36)?;
    let m2_reverse = LedcDriver::new(peripherals.ledc.channel3, driver4, peripherals.pins.gpio35)?;
    // enabled by default
    let m1_enable = peripherals.pins.gpio10;
    let m2_enable = peripherals.pins.gpio7;

    let mut motors = L293x::new(
        m1_forward, m1_reverse, m2_forward, m2_reverse, m1_enable, m2_enable,
    );

    // Encoder
    let mut m1_pin_a = peripherals.pins.gpio33;
    let mut m1_pin_b = peripherals.pins.gpio47;
    let mut m2_pin_a = peripherals.pins.gpio34;
    let mut m2_pin_b = peripherals.pins.gpio48;
    println!("setup encoder");
    let m1encoder = Encoder::new(peripherals.pcnt0, &mut m1_pin_a, &mut m1_pin_b)?;
    let m2encoder = Encoder::new(peripherals.pcnt1, &mut m2_pin_a, &mut m2_pin_b)?;

    // Encoder
    println!("setup encoder");
    let mut last_value = 0i32;
    motors.set_y2_duty_cycle_fully_off()?;
    motors.set_y4_duty_cycle_fully_off()?;
    motors.set_y1_duty_cycle_fully_off()?;
    motors.set_y3_duty_cycle_fully_off()?;

    let mut timer = TimerDriver::new(peripherals.timer00, &TimerConfig::new())?;

    block_on(async {
        let mut last_value1 = 0;
        let mut last_value2 = 0;
        loop {
            timer.delay(timer.tick_hz()).await.unwrap(); // Every second
            println!("every second");
            // Kd erstmal 0
            let value1 = m1encoder.get_value().unwrap();
            let value2 = m2encoder.get_value().unwrap();
            // encoderziel für drehung: 5647
            // TODO auch den KD wert normalisieren
            // Error: EncoderZiel - (linkerMotorEncoder - rechterMotorEncoder)
            let partial_error =
                K_P * ((2 * ENCODERZIEL - (value1 - value2)) as f32 / ENCODERZIEL as f32 - 1.0);
            let differential_error =
                K_D * -(((last_value1 - last_value2) - (value1 - value2)).abs()) as f32;
            let output = (partial_error + differential_error).clamp(-1.0, 1.0);

            last_value1 = value1;
            last_value2 = value2;
            turn_left_motor(&mut motors, output);
            turn_right_motor(&mut motors, output);
        }
    });

    println!("Starting duty-cycle loop");
    loop {
        println!("inside the loop");
        let value = m1encoder.get_value()?;
        if value != last_value {
            println!("value: {value}");
            last_value = value;
        }
        let value = m2encoder.get_value()?;
        if value != last_value {
            println!("value: {value}");
            last_value = value;
        }

        delay::Ets::delay_ms(300);
    }
}

// esp-idf encoder implementation using v4 pcnt api
mod encoder {
    use std::cmp::min;
    use std::sync::atomic::AtomicI32;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    use esp_idf_hal::gpio::AnyInputPin;
    use esp_idf_hal::gpio::InputPin;
    use esp_idf_hal::pcnt::*;
    use esp_idf_hal::peripheral::Peripheral;
    use esp_idf_sys::EspError;

    const LOW_LIMIT: i16 = -100;
    const HIGH_LIMIT: i16 = 100;

    pub struct Encoder<'d> {
        unit: PcntDriver<'d>,
        approx_value: Arc<AtomicI32>,
    }

    impl<'d> Encoder<'d> {
        pub fn new<PCNT: Pcnt>(
            pcnt: impl Peripheral<P = PCNT> + 'd,
            pin_a: impl Peripheral<P = impl InputPin> + 'd,
            pin_b: impl Peripheral<P = impl InputPin> + 'd,
        ) -> Result<Self, EspError> {
            let mut unit = PcntDriver::new(
                pcnt,
                Some(pin_a),
                Some(pin_b),
                Option::<AnyInputPin>::None,
                Option::<AnyInputPin>::None,
            )?;
            // The Idea here is that each "channel" is a pair of pins that are connected to the
            // encoder. Since CLK and DT will determine which rising/falling edge determines the
            // count, we will have a channel each for forward and backward movement.
            unit.channel_config(
                PcntChannel::Channel0,
                PinIndex::Pin0,
                PinIndex::Pin1,
                &PcntChannelConfig {
                    lctrl_mode: PcntControlMode::Reverse,
                    hctrl_mode: PcntControlMode::Keep,
                    pos_mode: PcntCountMode::Decrement,
                    neg_mode: PcntCountMode::Increment,
                    counter_h_lim: HIGH_LIMIT,
                    counter_l_lim: LOW_LIMIT,
                },
            )?;
            unit.channel_config(
                PcntChannel::Channel1,
                PinIndex::Pin1,
                PinIndex::Pin0,
                &PcntChannelConfig {
                    lctrl_mode: PcntControlMode::Reverse,
                    hctrl_mode: PcntControlMode::Keep,
                    pos_mode: PcntCountMode::Increment,
                    neg_mode: PcntCountMode::Decrement,
                    counter_h_lim: HIGH_LIMIT,
                    counter_l_lim: LOW_LIMIT,
                },
            )?;

            unit.set_filter_value(min(10 * 80, 1023))?;
            unit.filter_enable()?;

            let approx_value = Arc::new(AtomicI32::new(0));
            // unsafe interrupt code to catch the upper and lower limits from the encoder
            // and track the overflow in `value: Arc<AtomicI32>` - I plan to use this for
            // a wheeled robot's odomerty
            unsafe {
                let approx_value = approx_value.clone();
                unit.subscribe(move |status| {
                    let status = PcntEventType::from_repr_truncated(status);
                    if status.contains(PcntEvent::HighLimit) {
                        approx_value.fetch_add(HIGH_LIMIT as i32, Ordering::SeqCst);
                    }
                    if status.contains(PcntEvent::LowLimit) {
                        approx_value.fetch_add(LOW_LIMIT as i32, Ordering::SeqCst);
                    }
                })?;
            }
            unit.event_enable(PcntEvent::HighLimit)?;
            unit.event_enable(PcntEvent::LowLimit)?;
            unit.counter_pause()?;
            unit.counter_clear()?;
            unit.counter_resume()?;

            Ok(Self { unit, approx_value })
        }

        pub fn get_value(&self) -> Result<i32, EspError> {
            let value =
                self.approx_value.load(Ordering::Relaxed) + self.unit.get_counter_value()? as i32;
            Ok(value)
        }
    }
}

fn turn_right_motor(
    mut motors: &mut L293x<
        LedcDriver<'_>,
        LedcDriver<'_>,
        LedcDriver<'_>,
        LedcDriver<'_>,
        Gpio10,
        Gpio7,
    >,
    direction: f32,
) -> anyhow::Result<()> {
    if direction < 0.0 {
        motors.set_y1_duty_cycle_fully_off()?;
        motors.set_y2_duty_cycle_percent((direction * (-100.0)) as u8)?;
    } else if direction > 0.0 {
        motors.set_y1_duty_cycle_percent((direction * 100.0) as u8)?;
        motors.set_y2_duty_cycle_fully_off()?;
    } else {
        motors.set_y1_duty_cycle_fully_off()?;
        motors.set_y2_duty_cycle_fully_off()?;
    }
    Ok(())
}

/// takes -1 to 1
fn turn_left_motor(
    motors: &mut L293x<
        LedcDriver<'_>,
        LedcDriver<'_>,
        LedcDriver<'_>,
        LedcDriver<'_>,
        Gpio10,
        Gpio7,
    >,
    direction: f32,
) -> anyhow::Result<()> {
    if direction < 0.0 {
        motors.set_y3_duty_cycle_percent((direction * (-100.0)) as u8)?;
        motors.set_y4_duty_cycle_fully_off()?;
    } else if 0.0 < direction {
        motors.set_y3_duty_cycle_fully_off()?;
        motors.set_y4_duty_cycle_percent((direction * 100.0) as u8)?;
    } else {
        motors.set_y3_duty_cycle_fully_off()?;
        motors.set_y4_duty_cycle_fully_off()?;
    }
    Ok(())
}
