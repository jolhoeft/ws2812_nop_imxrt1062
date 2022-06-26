#![feature(const_fn_floating_point_arithmetic)]
// # Use ws2812 leds on imxrt1062 boards
//
// - For usage with `smart-leds`
// - Implements the `SmartLedsWrite` trait
//
// ## WARNING
// WS2812s operate at 5v.
// IMXRT1062 pads operate at 3v and are not 5v tolerant.
// You will need a logic-level converter and some care in wiring.
//
//
// ## Why?
//
// Because I'm apparently too stupid to get a periodic timer working with ws2812-timer-delay.
//
// ## How?
//
// This library uses a simple `nop` loop in assembly to wait.
//
// WS2812s read inputs with cycles about 333ns long.
// Write 3 bytes (GRB | Green, Red, Blue) _**per LED**_, then "latch" (set low) for 6us to 250us depending on your model.
// Writing a single bit entails a three-bit message that looks like `[1, x, 0]`, where x is the bit you want to write.
// That all is to say, to write a bit of `1` to an LED you must send `[1, 1, 0]`, waiting 333ns between each bit.
//
// Despite WS2812 cycles allegedly being 333ns,
// some of the time constraints are very tight, and others are very loose.
//
// Find out more about the [timing constraints of WS2812s](https://wp.josh.com/2014/05/13/ws2812-neopixels-are-not-so-finicky-once-you-get-to-know-them/).
//
// P.S. I have the latch hard coded to 6us. If you need 250us shoot me an email or clone the project.
//
// ## Example using teensy4-bsp
//
// ```rust
//
// #[cortex_m_rt::entry]
// fn main() -> ! {
//     ...
//
//     let mut data: [RGB8; 3] = [RGB8::default(); 3];
//     let empty: [RGB8; 3] = [RGB8::default(); 3];
//
//     data[0] = RGB8 {
//         r: 0x10,
//         g: 0x00,
//         b: 0x00,
//     };
//     data[1] = RGB8 {
//         r: 0x00,
//         g: 0x10,
//         b: 0x00,
//     };
//     data[2] = RGB8 {
//         r: 0x00,
//         g: 0x00,
//         b: 0x10,
//     };
//
//     configure(&mut pins.p2,{
//         Config::zero()
//             .set_drive_strength(DriveStrength::R0_7)
//             .set_speed(Speed::Max)
//             .set_slew_rate(SlewRate::Fast)
//     });
//     let pin = GPIO::new(pins.p2).output();
//
//     let mut ws = ws2812_nop_imxrt1062::Ws2812::new(pin, 600.0);
//
//     loop {
//         ws.write(data.iter().cloned()).unwrap();
//         systick.delay_ms(500);
//
//         ws.write(empty.iter().cloned()).unwrap();
//         systick.delay_ms(500);
//     }
// }
// ```
#![no_std]

use core::arch::asm;
use embedded_hal::digital::v2::OutputPin;

use smart_leds_trait::{SmartLedsWrite, RGB8};

const CYCLES_PER_LOOP: f32 = 3.0;

const fn n_loops_at(ns: f32, mhz: f32) -> i32 {
    (ns / ((1000.0 / mhz) * CYCLES_PER_LOOP)) as i32
}

pub struct Ws2812<PIN> {
    pub pin: PIN,
    pub frequency_mhz: f32,
}

impl<PIN> Ws2812<PIN>
where
    PIN: OutputPin,
{
    /// The timer has to already run at with a frequency of 3 MHz
    pub fn new(mut pin: PIN, frequency_mhz: f32) -> Ws2812<PIN> {
        pin.set_low().ok();
        Self { pin, frequency_mhz }
    }

    /// Wait for (ideally) 333ns
    #[inline(always)]
    pub fn wait(&self, loops: i32) {
        unsafe {
            asm!(
                "mov     r2, {0}",

                "2:",
                    "nop",
                    "nop",
                    "subs     r2, 1",
                    "cmp      r2, 0",
                    "bne      2b",

                in(reg) loops
            )
        }
    }

    fn write_bit(&mut self, bit: bool) {
        if bit {
            self.pin.set_high().ok();
            self.wait(n_loops_at(700.0, self.frequency_mhz));
            self.pin.set_low().ok();
            self.wait(n_loops_at(350.0, self.frequency_mhz));
        } else {
            self.pin.set_high().ok();
            self.wait(n_loops_at(300.0, self.frequency_mhz));
            self.pin.set_low().ok();
            self.wait(n_loops_at(666.0, self.frequency_mhz));
        }
    }

    fn write_byte(&mut self, mut data: u8) {
        for _ in 0..8 {
            self.write_bit((data & 0x80) != 0);
            data <<= 1;
        }
    }
}

impl<PIN> SmartLedsWrite for Ws2812<PIN>
where
    PIN: OutputPin,
{
    type Error = ();
    type Color = RGB8;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
    where
        T: Iterator<Item = I>,
        I: Into<Self::Color>,
    {
        for item in iterator {
            let item = item.into();
            self.write_byte(item.g);
            self.write_byte(item.r);
            self.write_byte(item.b);
        }

        // TODO: add feature cfg for setting wait time to 250us
        self.wait(n_loops_at(6000.0, self.frequency_mhz));
        Ok(())
    }
}
