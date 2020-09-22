#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m as _;
use cortex_m_rt::{entry, exception};

use stm32f4xx_hal as hal;

use hal::prelude::*;
use hal::stm32;

use core::sync::atomic::{AtomicBool, Ordering};

use rtt_target::{rprintln, rtt_init_print};

static TOGGLE_LED: AtomicBool = AtomicBool::new(false);

#[exception]
fn SysTick() {
    rprintln!("Trying to print");
    TOGGLE_LED.store(true, Ordering::Release);
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let p = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let gpioc = p.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output();

    // Set up the system clock. We want to run at 48MHz for this one.
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(16.mhz()).freeze();

    // configure SysTick to generate an exception every second
    let mut syst = cp.SYST;

    // Set up system timer to trigger interrupt when the counter reaches
    // set_reload value?
    // NOTE that counter increments by 1 per CPU clock tick and wraps at
    // 0x00ffffff, so we cannot run the CPU clock at 48 MHz if we want to sleep for a full second.
    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    syst.set_reload(clocks.sysclk().0);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();

    // Create a delay abstraction based on SysTick
    //let mut delay = hal::delay::Delay::new(syst, clocks);

    rprintln!("Entering the loop");

    let mut i = 0;
    loop {
        // sleep
        cortex_m::asm::wfi();
        if TOGGLE_LED.swap(false, Ordering::AcqRel) {
            led.toggle().unwrap();
        }

        i = i + 1;
        if i > 10 {
            // We break the loop to stop MCU from sleeping too hard.
            // This will allow us to flash with ST-Link again.
            break;
        }
    }
    loop {}
}
