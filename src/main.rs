//! Drives an OLED display
#![no_std]
#![no_main]
use bsp::hal::Adc;
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Baseline;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    text::Text,
};
use embedded_hal::adc::OneShot;
use embedded_hal::digital::v2::OutputPin;
use embedded_time::{fixed_point::FixedPoint, rate::Extensions};
use panic_probe as _;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio, pac,
    sio::Sio,
    spi,
    watchdog::Watchdog,
};
use ssd1309::{prelude::GraphicsMode, Builder};

// all values below are microseconds
const WAIT_AFTER_WATERING: u32 = 30000;
const WATERING: u32 = 1000;
const SLEEP_AFTER_INFO: u32 = 1000;
const MEASUREMENT_CYCLE: u32 = 10000;
const HUMIDITY_THRESHOLD: i32 = 3000; // 3500 -> dry | 2000 -> wet

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Setup pin for relay (water pump)
    let mut waterpump_on = pins.gpio0.into_push_pull_output();
    waterpump_on.set_high().unwrap();

    let mut led_pin = pins.led.into_push_pull_output();

    // These are implicitly used by the spi driver if they are in the correct mode
    let _spi_sclk = pins.gpio2.into_mode::<gpio::FunctionSpi>(); // scl
    let _spi_mosi = pins.gpio3.into_mode::<gpio::FunctionSpi>(); // sda
    let _spi_miso = pins.gpio4.into_mode::<gpio::FunctionSpi>();
    let spi_dc = pins.gpio5.into_push_pull_output();
    let mut reset = pins.gpio6.into_push_pull_output();

    // Create an SPI driver instance for the SPI0 device
    let spi = spi::Spi::<_, _, 8>::new(pac.SPI0);

    // Exchange the uninitialised SPI driver for an initialised one
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        400000_u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );
    let spi_interface = SPIInterfaceNoCS::new(spi, spi_dc);

    let mut disp: GraphicsMode<_> = Builder::new().connect(spi_interface).into();

    disp.reset(&mut reset, &mut delay).unwrap();

    disp.init().unwrap();
    disp.flush().unwrap();

    let test_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    // Setup ADC (moisture sensor)
    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let mut adc_pin_0 = pins.gpio26.into_floating_input();
    // first read seems off. This will be ignored:
    let mut pin_adc_counts: i32 = adc.read(&mut adc_pin_0).unwrap();
    delay.delay_ms(1000);

    let mut buffer = ryu::Buffer::new();

    let mut text = "";
    loop {
        led_pin.set_high().unwrap();
        pin_adc_counts = adc.read(&mut adc_pin_0).unwrap();
        text = buffer.format(pin_adc_counts as f32);
        Text::with_baseline(text, Point::zero(), test_style, Baseline::Top)
            .draw(&mut disp)
            .unwrap();
        disp.flush().unwrap();
        delay.delay_ms(SLEEP_AFTER_INFO);

        if pin_adc_counts > HUMIDITY_THRESHOLD {
            disp.clear();
            Text::with_baseline(
                "value > 2300. watering...",
                Point::zero(),
                test_style,
                Baseline::Top,
            )
            .draw(&mut disp)
            .unwrap();
            disp.flush().unwrap();
            waterpump_on.set_low().unwrap();
            delay.delay_ms(WATERING);
            waterpump_on.set_high().unwrap();

            disp.clear();
            Text::with_baseline(
                "Finished watering.\nWaiting 30 seconds...",
                Point::zero(),
                test_style,
                Baseline::Top,
            )
            .draw(&mut disp)
            .unwrap();
            disp.flush().unwrap();
            delay.delay_ms(WAIT_AFTER_WATERING);
        }
        disp.clear();
        led_pin.set_low().unwrap();
        delay.delay_ms(MEASUREMENT_CYCLE);
    }
}
