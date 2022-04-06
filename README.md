# Use ws2812 leds on imxrt1062 boards

- For usage with `smart-leds`
- Implements the `SmartLedsWrite` trait

## WARNING
WS2812s operate at 5v.
IMXRT1062 pads operate at 3v and are not 5v tolerant.
You will need a logic-level converter and some care in wiring.


## Why?

Because I'm apparently too stupid to get a periodic timer working with ws2812-timer-delay.

## How?

This library uses a simple for loop with no-ops inside.
How many loops is determined by a magic number.

The default magic_number is 50.

WS2812s read inputs with cycles about 333ns long.
Write 3 bytes (GRB | Green, Red, Blue) _**per LED**_, then "latch" (set low) for 6us to 250us depending on your model.
Writing a single bit entails a three-bit message that looks like `[1, x, 0]`, where x is the bit you want to write.
That all is to say, to write a bit of `1` to an LED you must send `[1, 1, 0]`, waiting 333ns between each bit.


As for the magic number, 600MHz a cycle should take 1.667 nanoseconds.
The loop inside takes about 4-6 cycles, depending on your magic number (whether the loop is unrolled).
This could be improved to be invariable by hand-rolling a loop in assembly.

In theory that means 333ns should be about 50 unrolled loops.

WS2812's have a pretty wide range of timing constraints,
so this will work fine even with a clock rate of 720MHz.

Calculate your own magic number: `333/((1000/CLOCK_SPEED_MHZ) * 4)`
e.g.: 333/((1000/600) * 4) = ~50

Find out more about the [timing constraints of WS2812s](https://wp.josh.com/2014/05/13/ws2812-neopixels-are-not-so-finicky-once-you-get-to-know-them/).

## Example using teensy4-bsp

```rust

#[cortex_m_rt::entry]
fn main() -> ! {
    ...

    let mut data: [RGB8; 3] = [RGB8::default(); 3];
    let empty: [RGB8; 3] = [RGB8::default(); 3];

    data[0] = RGB8 {
        r: 0x10,
        g: 0x00,
        b: 0x00,
    };
    data[1] = RGB8 {
        r: 0x00,
        g: 0x10,
        b: 0x00,
    };
    data[2] = RGB8 {
        r: 0x00,
        g: 0x00,
        b: 0x10,
    };

    configure(&mut pins.p2,{
        Config::zero()
            .set_drive_strength(DriveStrength::R0_7)
            .set_speed(Speed::Max)
            .set_slew_rate(SlewRate::Fast)
    });
    let pin = GPIO::new(pins.p2).output();

    let mut ws = ws2812_nop_imxrt1062::Ws2812::new(pin);

    loop {
        ws.write(data.iter().cloned()).unwrap();
        systick.delay_ms(500);

        ws.write(empty.iter().cloned()).unwrap();
        systick.delay_ms(500);
    }
}
```
