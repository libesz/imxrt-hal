//! Audio playback using sai peripheral and imxrt-hal.
//!
//! Plays back a simple 440Hz (A note) simple square wave tone with the SAI peripheral
//! to a Wolfson WM8960 codec on a number of the EVK boards.
//!
//! The audio stream itself is expected to be a 48000Hz 16bit stereo signal.

#![no_main]
#![no_std]

/// Half of a sine wave (0 to pi)
/// Can be used to generate a full sine wave by inverting (-1 * SIN_LUT[X]);
const SIN_LUT: [u16; 256] = [
    0, 402, 804, 1206, 1608, 2009, 2411, 2811, 3212, 3612, 4011, 4410, 4808, 5205, 5602, 5998,
    6393, 6787, 7180, 7571, 7962, 8351, 8740, 9127, 9512, 9896, 10279, 10660, 11039, 11417, 11793,
    12167, 12540, 12910, 13279, 13646, 14010, 14373, 14733, 15091, 15447, 15800, 16151, 16500,
    16846, 17190, 17531, 17869, 18205, 18538, 18868, 19195, 19520, 19841, 20160, 20475, 20788,
    21097, 21403, 21706, 22006, 22302, 22595, 22884, 23170, 23453, 23732, 24008, 24279, 24548,
    24812, 25073, 25330, 25583, 25833, 26078, 26320, 26557, 26791, 27020, 27246, 27467, 27684,
    27897, 28106, 28311, 28511, 28707, 28899, 29086, 29269, 29448, 29622, 29792, 29957, 30118,
    30274, 30425, 30572, 30715, 30853, 30986, 31114, 31238, 31357, 31471, 31581, 31686, 31786,
    31881, 31972, 32058, 32138, 32214, 32286, 32352, 32413, 32470, 32522, 32568, 32610, 32647,
    32679, 32706, 32729, 32746, 32758, 32766, 32767, 32766, 32758, 32746, 32729, 32706, 32679,
    32647, 32610, 32568, 32522, 32470, 32413, 32352, 32286, 32214, 32138, 32058, 31972, 31881,
    31786, 31686, 31581, 31471, 31357, 31238, 31114, 30986, 30853, 30715, 30572, 30425, 30274,
    30118, 29957, 29792, 29622, 29448, 29269, 29086, 28899, 28707, 28511, 28311, 28106, 27897,
    27684, 27467, 27246, 27020, 26791, 26557, 26320, 26078, 25833, 25583, 25330, 25073, 24812,
    24548, 24279, 24008, 23732, 23453, 23170, 22884, 22595, 22302, 22006, 21706, 21403, 21097,
    20788, 20475, 20160, 19841, 19520, 19195, 18868, 18538, 18205, 17869, 17531, 17190, 16846,
    16500, 16151, 15800, 15447, 15091, 14733, 14373, 14010, 13646, 13279, 12910, 12540, 12167,
    11793, 11417, 11039, 10660, 10279, 9896, 9512, 9127, 8740, 8351, 7962, 7571, 7180, 6787, 6393,
    5998, 5602, 5205, 4808, 4410, 4011, 3612, 3212, 2811, 2411, 2009, 1608, 1206, 804, 402,
];

/// Generate a sine wave sample
fn sine(t: u32) -> u16 {
    let p = t % 512;
    let s = SIN_LUT[(p % 256) as usize];
    if p < 256 {
        (32768 + s) / 2
    } else {
        (32768 - s) / 2
    }
}

/// Generate a square wave sample
fn square(t: u32) -> u16 {
    if (t % 128) > 64 {
        32767
    } else {
        0
    }
}

#[rtic::app(device = board, peripherals = false, dispatchers = [BOARD_SWTASK0])]
mod app {

    //
    // Configure the demo below.
    //

    /// Change me to change how log messages are serialized
    /// and transported.
    const FRONTEND: board::logging::Frontend = board::logging::Frontend::Log;
    /// This is a function of your board. Want to change it? Change it right
    /// here to explore different example code paths.
    const BACKEND: board::logging::Backend = board::logging::BACKEND;

    /// When using the LPUART backend, you want to occasionally poll
    /// for new log message. This interval (milliseconds) describes how
    /// long you should wait in between polls.
    ///
    /// This constant does nothing when the USBD backend is used, since
    /// the USBD backend uses its own timer for this purpose.
    const LPUART_POLL_INTERVAL_MS: u32 = board::PIT_FREQUENCY / 1_000 * 4;
    /// How frequently (milliseconds) should we make a log message?
    ///
    /// Decrease this constant to log more frequently.
    const MAKE_LOG_INTERVAL_MS: u32 = board::PIT_FREQUENCY / 1_000 * 250;

    /// How frequently (milliseconds) should we poll audio
    const AUDIO_POLL_MS: u32 = 1000 * (board::PIT_FREQUENCY / 1_000);

    use crate::{sine, square};
    use imxrt_hal as hal;

    type SaiTx = hal::sai::Tx<1, 16, 2, hal::sai::PackingNone>;

    //
    // End configurations.
    //

    #[local]
    struct Local {
        /// Toggle when we make a log.
        led: board::Led,
        /// This timer tells us how frequently to poll
        /// for logs. It's only used with the LPUART
        /// logging backend.
        poll_log: hal::pit::Pit<1>,
        /// This timer tells us how frequently to generate
        /// logs. It's always used.
        make_log: hal::pit::Pit<2>,

        /// This timer tells us how frequently work on audio.
        audio_pit: hal::pit::Pit<3>,

        /// Sample counter for the wave generation
        counter: u32,
    }

    #[shared]
    struct Shared {
        /// Serial audio interface
        sai1_tx: SaiTx,
        poller: board::logging::Poller,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let mut cortex_m = cx.core;
        let (
            board::Common {
                pit: (_, mut poll_log, mut make_log, mut audio_pit),
                usb1,
                usbnc1,
                usbphy1,

                mut dma,
                ..
            },
            board::Specifics {
                led, sai1, console, ..
            },
        ) = board::new();

        // We only need the extra timer when the LPUART backend is used.
        // The USBD backend uses the USB peripheral's internal timer to
        // track time for us.
        if BACKEND == board::logging::Backend::Lpuart {
            poll_log.set_load_timer_value(LPUART_POLL_INTERVAL_MS);
            poll_log.set_interrupt_enable(true);
            poll_log.enable();
        } else {
            poll_log.disable();
        }

        make_log.set_load_timer_value(MAKE_LOG_INTERVAL_MS);
        make_log.set_interrupt_enable(true);
        make_log.enable();

        let usbd = hal::usbd::Instances {
            usb: usb1,
            usbnc: usbnc1,
            usbphy: usbphy1,
        };

        let dma_a = dma[board::BOARD_DMA_A_INDEX].take().unwrap();
        let poller = board::logging::init(FRONTEND, BACKEND, console, dma_a, usbd);

        let miez = hal::sai::SaiConfig::i2s(hal::sai::bclk_div(64));
        let (Some(sai1_tx), Some(sai1_rx)) = sai1.split(&miez)
        else {
            panic!("Unexpected return from sai split");
        };

        let mut sai1_tx: SaiTx = sai1_tx;

        let regs = sai1_tx.reg_dump();
        defmt::println!(
            "Regdump of config: TCR1: {:b}, TCR2 {:b}, TCR3 {:b}, TCR4 {:b}, TCR5 {:b}",
            regs[0],
            regs[1],
            regs[2],
            regs[3],
            regs[4]
        );

        cortex_m.DCB.enable_trace();
        cortex_m::peripheral::DWT::unlock();
        cortex_m.DWT.enable_cycle_counter();

        audio_pit.set_load_timer_value(AUDIO_POLL_MS);
        audio_pit.set_interrupt_enable(true);
        audio_pit.enable();

        let mut counter: u32 = 0;
        for _i in 0..31 {
            sai1_tx.write_frame(0, [sine(counter), square(counter)]);
            counter += 1;
        }
        sai1_tx.set_interrupts(
            hal::sai::Interrupts::FIFO_WARNING | hal::sai::Interrupts::FIFO_REQUEST,
        );
        sai1_tx.set_enable(true);

        (
            Shared {
                sai1_tx, poller,
            },
            Local {
                led: led,
                poll_log,
                make_log,
                audio_pit,
                counter,
            },
        )
    }

    #[task(binds = BOARD_SAI1, shared = [sai1_tx], local = [counter], priority = 2)]
    fn sai1_interrupt(mut cx: sai1_interrupt::Context) {
        let sai1_interrupt::LocalResources { counter, .. } = cx.local;

        cx.shared.sai1_tx.lock(|sai1_tx| {
            while sai1_tx.status().contains(hal::sai::Status::FIFO_REQUEST) {
                sai1_tx.write_frame(0, [sine(*counter), square(*counter)]);
                *counter = (*counter).wrapping_add(1);
            }
        });
    }

    /// This interrupt fires
    ///
    /// - when log messages have been written (to the USB host).
    /// - every few milliseconds; check the imxrt-log docs for the
    ///   specific number.
    #[task(binds = BOARD_USB1, priority = 1)]
    fn usb_interrupt(_: usb_interrupt::Context) {
        poll_logger::spawn().unwrap();
    }

    /// This interrupt fires
    ///
    /// - when log messages have been written.
    ///
    /// Notice how there's no "periodic" or "time" component here.
    /// When using the LPUART backend, you should use another time
    /// source, or a polling loop, to make sure poll periodically
    /// happens. Without this, you won't see your log messages.
    #[task(binds = BOARD_DMA_A, priority = 1)]
    fn dma_interrupt(_: dma_interrupt::Context) {
        poll_logger::spawn().unwrap();
    }

    /// Actually performs the poll call.
    #[task(shared = [poller], priority = 2)]
    async fn poll_logger(mut cx: poll_logger::Context) {
        cx.shared.poller.lock(|poller| poller.poll());
    }

    #[task(binds = BOARD_PIT, shared = [sai1_tx], local = [audio_pit, led, poll_log, make_log, log_counter: u32 = 0], priority = 1)]
    fn pit_interrupt(mut cx: pit_interrupt::Context) {
        let pit_interrupt::LocalResources {
            audio_pit,
            poll_log,
            make_log,
            led,
            log_counter,
            ..
        } = cx.local;

        //led.toggle();
        while audio_pit.is_elapsed() {
            audio_pit.clear_elapsed();
        }

        let (status, write_pos, read_pos) = cx.shared.sai1_tx.lock(|sai1_tx| {
            let status = sai1_tx.status();
            let (write_pos, read_pos) = sai1_tx.fifo_position(0);
            (status, write_pos, read_pos)
        });

        log::info!(
            "Audio synthesis tx status {:#x}, fifo underrun? {}, write pos {}, read pos {}",
            status.bits(),
            status.contains(hal::sai::Status::FIFO_ERROR),
            write_pos,
            read_pos,
        );

        // Is it time for us to poll the logger?
        // This only happens for the LPUART backend.
        if poll_log.is_elapsed() {
            while poll_log.is_elapsed() {
                poll_log.clear_elapsed();
            }
            poll_logger::spawn().unwrap();
        }

        // // Is it time for us to send a new log message?
        // if make_log.is_elapsed() {
        //     led.toggle();
        //     while make_log.is_elapsed() {
        //         make_log.clear_elapsed();
        //     }

        //     let count = cycles(|| {
        //         log::info!(
        //             "Hello from the log framework over {BACKEND:?}! The count is {log_counter}"
        //         )
        //     });
        //     log::info!(
        //         "That last message took {count} cycles to be copied into the logging buffer"
        //     );

        //     let count = cycles(|| {
        //         defmt::println!(
        //             "Hello from defmt over {}! The count is {=u32}",
        //             BACKEND,
        //             log_counter
        //         )
        //     });
        //     defmt::println!(
        //         "That last message took {=u32} cycles to be copied into the logging buffer",
        //         count
        //     );

        //     *log_counter += 1;
        // }
    }

    /// Count the clock cycles required to execute `f`
    fn cycles<F: FnOnce()>(f: F) -> u32 {
        let start = cortex_m::peripheral::DWT::cycle_count();
        f();
        let end = cortex_m::peripheral::DWT::cycle_count();
        end - start
    }
}
