#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m as _;
use cortex_m_rt::{entry, exception};

use stm32f4xx_hal as hal;

use core::cell::RefCell;
use hal::interrupt;

use hal::prelude::*;
use hal::stm32;

use hal::gpio::ExtiPin;

use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m::interrupt::Mutex;

use rtt_target::{rprintln, rtt_init_print};

static TOGGLE_LED: AtomicBool = AtomicBool::new(false);
static MUTEX_EXTI: Mutex<RefCell<Option<hal::stm32::EXTI>>> =
    Mutex::new(RefCell::new(Option::None));

use stm32f4xx_hal::gpio::gpioa::PA0;
use stm32f4xx_hal::gpio::{Input, PullUp};

static MUTEX_KEY: Mutex<RefCell<Option<PA0<Input<PullUp>>>>> =
    Mutex::new(RefCell::new(Option::None));

static KEY_PUSHED: AtomicBool = AtomicBool::new(false);

#[interrupt]
fn EXTI0() {
    // Enter critical section and unset interrupt EXTI0 pending bit
    cortex_m::interrupt::free(|cs| {
        // enter critical section
        let exti = MUTEX_EXTI.borrow(cs).borrow(); // acquire Mutex
        if exti.is_none() {
            return;
        }

        exti.as_ref()
            .unwrap() // unwrap RefCell
            .pr
            .modify(|_, w| w.pr0().set_bit()); // clear the EXTI line 0 pending bit

        let key_ref = MUTEX_KEY.borrow(cs).borrow();
        let key = key_ref.as_ref().unwrap();

        let pressed = key.is_low().unwrap();
        let old_state = KEY_PUSHED.compare_and_swap(!pressed, pressed, Ordering::SeqCst);
        if old_state != pressed {
            if pressed {
                // Key state changed to Down
                rprintln!("Button is down");
                TOGGLE_LED.store(false, Ordering::SeqCst);
            } else {
                // Key state changed to Up
                rprintln!("Button is up");
                TOGGLE_LED.store(true, Ordering::SeqCst);
            }
        }
    });
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let p = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let mut syscfg = p.SYSCFG;
    let mut exti = p.EXTI;

    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

    // configure SysTick to generate an exception every second
    let syst = cp.SYST;

    // Create a delay abstraction based on SysTick
    let mut _delay = hal::delay::Delay::new(syst, clocks);

    // Unmask EXTI0 interrupt
    exti.imr.modify(|_, w| w.mr0().set_bit());

    // Setup Key to push interrupts
    let gpioa = p.GPIOA.split();
    let mut key = gpioa.pa0.into_pull_up_input();
    key.trigger_on_edge(&mut exti, hal::gpio::Edge::RISING_FALLING);
    key.make_interrupt_source(&mut syscfg);

    // If key is held down at boot we enter a busy loop (no wait-for-interrupts)
    // to allow ST-Link flashing
    if key.is_low().unwrap() {
        loop {}
    }

    // Setup LED
    let gpioc = p.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output();
    // Turn on LED to indicate we've "started up".
    led.set_low().unwrap();
    let mut led_state = true;

    // Enable EXTI0 Interrupt
    unsafe {
        cortex_m::peripheral::NVIC::unmask(stm32f4xx_hal::interrupt::EXTI0);
    }

    // Enter critical section and store EXTI in global mutex
    cortex_m::interrupt::free(|cs| {
        MUTEX_EXTI.borrow(cs).replace(Option::Some(exti));
        MUTEX_KEY.borrow(cs).replace(Option::Some(key));
    });

    rprintln!("Entering the loop");
    loop {
        // Wait for interrupts
        cortex_m::asm::wfi();

        if TOGGLE_LED.load(Ordering::SeqCst) != led_state {
            led_state = !led_state;
            led.toggle().unwrap();
        }
    }
}
