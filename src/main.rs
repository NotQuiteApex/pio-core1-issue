//! Firmware for JukeBox

#![no_std]
#![no_main]

mod st7789;

use panic_probe as _;
use rp_pico::hal::{
    clocks::init_clocks_and_plls,
    gpio::{DynPinId, FunctionPio1, FunctionSioOutput, Pin, PullDown},
    multicore::{Multicore, Stack},
    pac::Peripherals,
    pio::PIOExt,
    rom_data::reset_to_usb_boot,
    sio::Sio,
    watchdog::Watchdog,
    Timer,
};
use rp_pico::{entry, Pins};
use st7789::St7789;

#[allow(unused_imports)]
use defmt::*;
use defmt_rtt as _;

static mut CORE1_STACK: Stack<4096> = Stack::new();

#[entry]
fn main() -> ! {
    info!("starting!");

    // set up hardware interfaces
    let mut pac = Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();
    let mut sio = Sio::new(pac.SIO);
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let core1 = &mut mc.cores()[1];

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // set up timers
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // reset_to_usb_boot(0, 0);

    // core 1 event loop (GPIO)
    core1
        .spawn(unsafe { &mut CORE1_STACK.mem }, move || {
            // LABEL: MOVE CODE HERE TO TEST CORE1

            // * CODE
            let screen_pins: (
                Pin<DynPinId, FunctionPio1, PullDown>,      // data
                Pin<DynPinId, FunctionPio1, PullDown>,      // clock
                Pin<DynPinId, FunctionSioOutput, PullDown>, // cs
                Pin<DynPinId, FunctionSioOutput, PullDown>, // dc
                Pin<DynPinId, FunctionSioOutput, PullDown>, // rst
                Pin<DynPinId, FunctionSioOutput, PullDown>, // backlight
            ) = (
                pins.gpio21.into_function().into_dyn_pin().into_pull_type(), // data
                pins.gpio20.into_function().into_dyn_pin().into_pull_type(), // clock
                pins.gpio19.into_function().into_dyn_pin().into_pull_type(), // cs
                pins.gpio18.into_function().into_dyn_pin().into_pull_type(), // dc
                pins.gpio17.into_function().into_dyn_pin().into_pull_type(), // rst
                pins.gpio16.into_function().into_dyn_pin().into_pull_type(), // backlight
            );

            let (mut pio1, _, _, _, sm1) = pac.PIO1.split(&mut pac.RESETS);
            let mut st = St7789::new(
                &mut pio1,
                sm1,
                screen_pins.0,
                screen_pins.1,
                screen_pins.2,
                screen_pins.3,
                screen_pins.4,
                screen_pins.5,
                timer.count_down(),
            );

            st.init();
            // * CODE END

            loop {}
        })
        .expect("failed to start core1");

    // LABEL: MOVE CODE HERE TO TEST CORE0

    // main event loop (USB comms)
    loop {}
}
