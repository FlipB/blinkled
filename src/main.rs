#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m as _;
use cortex_m_rt::entry;

use stm32f4xx_hal as hal;

use hal::prelude::*;
use hal::stm32;

use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let p = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let gpioc = p.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output();

    // Set up the system clock. We want to run at 48MHz for this one.
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

    // Create a delay abstraction based on SysTick
    let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

    rprintln!("Entering the loop");

    loop {
        led.set_low().unwrap();
        delay.delay_ms(100_u32);
        led.set_high().unwrap();
        delay.delay_ms(100_u32);
    }
}
