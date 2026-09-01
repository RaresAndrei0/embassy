#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use embassy_executor::Spawner;
use nxp_pac::flexcomm::vals::Persel;
use nxp_pac::i2c;
use nxp_pac::i2c::vals::{Mstcontinue, Mstpending, Mststart, Mststate, Mststop};
use nxp_pac::iocon::vals::{PioDigimode, PioFunc, PioMode, PioOd, PioSlew};
use nxp_pac::{FLEXCOMM1, I2C1, IOCON, SYSCON};
use panic_halt as _;

const SLAVE_ADDR: u8 = 0x76;

#[entry]
fn main() -> ! {
    unsafe {
        let _p = embassy_nxp::init(Default::default());
        init_i2c1();

        write_byte(SLAVE_ADDR, 0xAA);
    }

    loop {
        cortex_m::asm::nop();
    }
}

unsafe fn init_i2c1() {
    SYSCON.ahbclkctrl0().modify(|w| w.set_iocon(true));

    IOCON.pio0(13).modify(|w| {
        w.set_func(PioFunc::Alt1);
        w.set_mode(PioMode::PullUp);
        w.set_od(PioOd::OpenDrain);
        w.set_digimode(PioDigimode::Digital);
        w.set_slew(PioSlew::Standard);
    });

    IOCON.pio0(14).modify(|w| {
        w.set_func(PioFunc::Alt1);
        w.set_mode(PioMode::PullUp);
        w.set_od(PioOd::OpenDrain);
        w.set_digimode(PioDigimode::Digital);
        w.set_slew(PioSlew::Standard);
    });

    SYSCON.ahbclkctrl1().modify(|w| w.set_fc(1, true));
    FLEXCOMM1.pselid().modify(|w| w.set_persel(Persel::I2c));
    I2C1.clkdiv().modify(|w| w.set_divval(9));
    I2C1.cfg().modify(|w| w.set_msten(true));
}

unsafe fn wait_ready() {
    while I2C1.stat().read().mstpending() != Mstpending::Pending {}
}

unsafe fn start(addr: u8, is_read: bool) {
    unsafe {
        wait_ready();
    }

    let rw_bit = if is_read { 1 } else { 0 };
    let tx = (addr << 1) | rw_bit;

    I2C1.mstdat().modify(|w| w.set_data(tx));
    I2C1.mstctl().modify(|w| w.set_mststart(Mststart::Start));

    unsafe {
        wait_ready();
    }

    let state = I2C1.stat().read().mststate();

    if state == Mststate::NackAddress || state == Mststate::NackData {
        loop {}
    }
}

unsafe fn write_byte(addr: u8, data: u8) {
    unsafe {
        start(addr, false);
    }

    unsafe {
        wait_ready();
    }
    I2C1.mstdat().modify(|w| w.set_data(data));
    I2C1.mstctl().modify(|w| w.set_mstcontinue(Mstcontinue::Continue));

    unsafe {
        wait_ready();
    }

    // send stop
    I2C1.mstctl().modify(|w| w.set_mststop(Mststop::Stop));
    unsafe {
        wait_ready();
    }
}
