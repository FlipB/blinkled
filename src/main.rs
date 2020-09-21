#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m as _;
use cortex_m_rt::entry;

use stm32f4xx_hal as hal;

use hal::prelude::*;
use hal::stm32;

use hal::spi::Spi;
use hal::spi::*;

use embedded_hal::digital::OutputPin;

use cc1101::{AddressFilter, Cc1101, Modulation, PacketLength, RadioMode, SyncMode};

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

    let mode0 = hal::spi::Mode {
        polarity: hal::spi::Polarity::IdleLow,
        phase: hal::spi::Phase::CaptureOnFirstTransition,
    };

    let gpioa = p.GPIOA.split();
    let sck = gpioa.pa5.into_alternate_af5();
    let miso = gpioa.pa6.into_alternate_af5();
    let mosi = gpioa.pa7.into_alternate_af5();

    let ss = gpioa.pa4.into_push_pull_output();
    let ss_pin = embedded_hal::digital::v1_compat::OldOutputPin::new(ss);

    let spi = Spi::spi1(
        p.SPI1,
        (sck, miso, mosi),
        mode0,
        50_000u32.hz(),
        clocks.clone(),
    );

    let mut cc1101 = Cc1101::new(spi, ss_pin).unwrap();

    cc1101.set_defaults().unwrap();
    cc1101.set_radio_mode(RadioMode::Receive).unwrap();
    cc1101.set_packet_length(PacketLength::Infinite).unwrap();
    cc1101.set_sync_mode(SyncMode::Disabled).unwrap();

    // 433_920_000u64
    cc1101.set_frequency(433_925_000u64).unwrap();

    cc1101
        .set_modulation(Modulation::BinaryFrequencyShiftKeying)
        .unwrap();
    //cc1101.set_sync_mode(SyncMode::MatchFull(0xD201)).unwrap();
    //cc1101.set_packet_length(PacketLength::Variable(17)).unwrap();

    /*
    cc1101
        .set_address_filter(AddressFilter::Device(0x3e))
        .unwrap();
        */

    cc1101.set_deviation(20_629).unwrap();
    cc1101.set_data_rate(4800).unwrap();
    //cc1101.set_chanbw(101_562).unwrap();

    let r = cc1101.get_hw_info().unwrap();

    // Create a delay abstraction based on SysTick
    let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

    rprintln!("Entering the loop");
    let mut addr: u8 = 0;
    let mut buf: [u8; 4] = [0; 4];
    loop {
        let len = cc1101.receive(&mut addr, &mut buf).unwrap();
        rprintln!("Received {:?}: {:?}", addr, buf.iter().take(len as usize));

        led.set_low().unwrap();
        delay.delay_ms(100_u32);
        led.set_high().unwrap();
        delay.delay_ms(100_u32);
    }
}
