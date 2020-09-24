use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;

use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use stm32f4xx_hal::rcc::Clocks;

/// System timer (SysTick) as a delay provider
pub struct NonblockingDelay {
    clocks: Clocks,
    syst: SYST,
    elapsed: u32,
    target: u32,
}

impl NonblockingDelay {
    /// Configures the system timer (SysTick) as a delay provider
    pub fn new(mut syst: SYST, clocks: Clocks) -> Self {
        syst.disable_interrupt();
        syst.set_clock_source(SystClkSource::Core);

        NonblockingDelay {
            syst,
            clocks,
            elapsed: 0,
            target: 0,
        }
    }

    /// Releases the system timer (SysTick) resource
    pub fn free(self) -> SYST {
        self.syst
    }

    pub fn is_done(&self) -> bool {
        self.elapsed >= self.target
    }

    /// called when interrupt signals the syst timer has wrapped
    pub fn interrupt(&mut self) {
        self.update_reload()
    }

    fn update_reload(&mut self) {
        if self.elapsed >= self.target {
            // timer is done
            self.syst.clear_current();
            self.syst.enable_counter();
            self.syst.disable_interrupt();
            self.syst.disable_counter();
        }

        let remaining = self.target - self.elapsed;

        // The SysTick Reload Value register supports values between 1 and 0x00FFFFFF.
        const MAX_RVR: u32 = 0x00FF_FFFF;

        let current_rvr = if remaining <= MAX_RVR {
            remaining
        } else {
            MAX_RVR
        };
        self.elapsed += current_rvr;
        self.syst.set_reload(current_rvr);
        self.syst.clear_current();
        self.syst.enable_counter();
        self.syst.enable_interrupt();
    }
}

impl DelayMs<u32> for NonblockingDelay {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us(ms * 1_000);
    }
}

impl DelayMs<u16> for NonblockingDelay {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(ms as u32);
    }
}

impl DelayMs<u8> for NonblockingDelay {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(ms as u32);
    }
}

impl DelayUs<u32> for NonblockingDelay {
    fn delay_us(&mut self, us: u32) {
        if !self.is_done() && self.elapsed > 0 {
            // timer is already running.
            panic!("Delay already in progress");
        }
        self.target = us * (self.clocks.sysclk().0 / 1_000_000);
        self.elapsed = 0;

        self.update_reload();
    }
}

impl DelayUs<u16> for NonblockingDelay {
    fn delay_us(&mut self, us: u16) {
        self.delay_us(us as u32)
    }
}

impl DelayUs<u8> for NonblockingDelay {
    fn delay_us(&mut self, us: u8) {
        self.delay_us(us as u32)
    }
}
