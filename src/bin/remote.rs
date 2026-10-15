//! Wheely remote: drive from a phone over Bluetooth LE.
//!
//! Advertises as "wheely" with the Nordic UART Service. Two ways to talk to it:
//!
//!   * Adafruit "Bluefruit Connect" app, Controller -> Control Pad.
//!     Arrows: up = forward, down = reverse, left/right = spin on the spot. Hold to move,
//!     release to stop. Buttons 1-4 pick the speed (slow .. full).
//!   * Any BLE UART terminal (Bluefruit's UART screen included), one letter per command:
//!     f b l r s = forward, back, left, right, stop; 1-4 = speed; the word "boot" reboots the
//!     Pico into BOOTSEL so it can be flashed without touching the button (handy on battery,
//!     where unplugging USB no longer resets it).
//!
//! Once a second wheely sends a status line back ("Forward 523mm", "!" appended when the
//! sonar is blocking forward motion). The UART screen shows it.
//!
//! Safety rules, in order of precedence:
//!   1. Phone disconnected (or never connected): motors coast.
//!   2. A movement held longer than MAX_HOLD stops by itself; press again.
//!   3. Sonar closer than STOP_MM (or sonar faulty/silent): forward is refused, reverse and
//!      turns still work so you can back out. Forward resumes after CLEAR_CONFIRM consecutive
//!      readings above GO_MM.
//!
//! Wiring as for `drive`: motor A (left) on GP0/GP1, motor B (right) on GP2/GP3, sonar on GP16/GP17.
//! Flash: hold BOOTSEL while plugging in, then `cargo run --release --bin remote`.

#![no_std]
#![no_main]

use core::cell::Cell;
use core::fmt::Write as _;

use cyw43::aligned_bytes;
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIO0, USB};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::pwm::Pwm;
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_rp::{bind_interrupts, dma};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Instant, Timer};
use panic_probe as _;
use static_cell::StaticCell;
use trouble_host::prelude::*;
use wheely::motor::{Motor, Speed, pwm_config};
use wheely::sonar::{PING_INTERVAL, SonarError, measure};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
});

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"wheely-remote"),
    embassy_rp::binary_info::rp_program_description!(c"phone remote over BLE, sonar guarded"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

// ---------------------------------------------------------------- tunables

/// Forward is refused when the obstacle is closer than this ...
const STOP_MM: u32 = 200;
/// ... and allowed again only once it is further than this (hysteresis).
const GO_MM: u32 = 300;
/// Sonar quiet for this long counts as a fault. Normal cadence is 10 Hz.
const SENSOR_TIMEOUT: Duration = Duration::from_millis(500);
/// Consecutive clear readings needed before forward is allowed again. One close reading
/// blocks at once. A close, angled object returns the odd missed echo, which otherwise
/// reads as "nothing ahead" and lets the motors twitch (seen on the bench 2026-10-15).
const CLEAR_CONFIRM: u8 = 3;
/// A movement command older than this is dropped. Guards against a phone that
/// died mid-press: the link itself can take several seconds to be declared dead.
const MAX_HOLD: Duration = Duration::from_secs(5);
/// How often the control loop re-evaluates motors.
const CONTROL_PERIOD: Duration = Duration::from_millis(50);
/// Speed presets for buttons 1-4, permille.
const PRESETS: [Speed; 4] = [300, 500, 750, 1000];
const DEFAULT_SPEED: Speed = 500;
/// Flip to -1 if a motor turns the wrong way.
const LEFT_SIGN: Speed = 1;
const RIGHT_SIGN: Speed = 1;

// ---------------------------------------------------------------- shared state

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Cmd {
    Stop,
    Forward,
    Reverse,
    Left,
    Right,
}

#[derive(Clone, Copy)]
struct Command {
    cmd: Cmd,
    speed: Speed,
    /// When `cmd` was last set. Drives MAX_HOLD.
    at: Instant,
}

/// What the phone wants. Written by the BLE side, read by the control loop.
static COMMAND: Mutex<CriticalSectionRawMutex, Cell<Command>> = Mutex::new(Cell::new(Command {
    cmd: Cmd::Stop,
    speed: DEFAULT_SPEED,
    at: Instant::from_ticks(0),
}));

/// Latest sonar reading and when it arrived. None until the first ping.
static SONAR: Mutex<CriticalSectionRawMutex, Cell<Option<(Result<u32, SonarError>, Instant)>>> =
    Mutex::new(Cell::new(None));

/// What the control loop is actually doing: effective command, sonar-blocked flag.
static STATUS: Mutex<CriticalSectionRawMutex, Cell<(Cmd, bool)>> = Mutex::new(Cell::new((Cmd::Stop, false)));

fn command() -> Command {
    COMMAND.lock(|c| c.get())
}

fn set_command(cmd: Cmd) {
    COMMAND.lock(|c| {
        let mut v = c.get();
        v.cmd = cmd;
        v.at = Instant::now();
        c.set(v);
    });
    log::info!("cmd {:?}", cmd);
}

/// A direction button was released: stop only if that is the direction we are holding.
/// (Press up, press right, release up should keep turning right.)
fn release(cmd: Cmd) {
    let stop = COMMAND.lock(|c| {
        let mut v = c.get();
        if v.cmd != cmd {
            return false;
        }
        v.cmd = Cmd::Stop;
        v.at = Instant::now();
        c.set(v);
        true
    });
    if stop {
        log::info!("cmd Stop (released {:?})", cmd);
    }
}

fn set_speed(speed: Speed) {
    COMMAND.lock(|c| {
        let mut v = c.get();
        v.speed = speed;
        c.set(v);
    });
    log::info!("speed {} permille", speed);
}

// ---------------------------------------------------------------- phone protocol

/// Bytes written to the NUS RX characteristic. Either a Bluefruit Connect control-pad
/// packet or plain text.
fn handle_rx(data: &[u8]) {
    // Control pad: '!' 'B' <button '1'..'8'> <'1' pressed | '0' released> <checksum>
    // checksum = bitwise NOT of the byte sum of the first four bytes.
    if data.len() == 5 && data[0] == b'!' && data[1] == b'B' {
        let sum = data[..4].iter().fold(0u8, |a, &b| a.wrapping_add(b));
        if !sum != data[4] {
            log::warn!("control pad packet with bad checksum: {:?}", data);
            return;
        }
        let pressed = data[3] == b'1';
        let dir = match data[2] {
            b'5' => Some(Cmd::Forward),
            b'6' => Some(Cmd::Reverse),
            b'7' => Some(Cmd::Left),
            b'8' => Some(Cmd::Right),
            _ => None,
        };
        match (dir, pressed) {
            (Some(d), true) => set_command(d),
            (Some(d), false) => release(d),
            (None, true) => {
                if let Some(i) = (data[2] as char).to_digit(10).filter(|d| (1..=4).contains(d)) {
                    set_speed(PRESETS[i as usize - 1]);
                }
            }
            (None, false) => {}
        }
        return;
    }

    // "boot": reboot into BOOTSEL for flashing. Everything resets, so the motors coast.
    if data.trim_ascii() == b"boot" {
        log::info!("rebooting into BOOTSEL");
        embassy_rp::rom_data::reset_to_usb_boot(0, 0);
    }

    // Plain text: one letter per command, anything else ignored.
    for &b in data {
        match b.to_ascii_lowercase() {
            b'f' => set_command(Cmd::Forward),
            b'b' => set_command(Cmd::Reverse),
            b'l' => set_command(Cmd::Left),
            b'r' => set_command(Cmd::Right),
            b's' => set_command(Cmd::Stop),
            b'1'..=b'4' => set_speed(PRESETS[(b - b'1') as usize]),
            b'\r' | b'\n' | b' ' => {}
            other => log::info!("ignored byte 0x{:02x}", other),
        }
    }
}

// ---------------------------------------------------------------- tasks

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>, cyw43::Cyw43439>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn sonar_task(mut trig: Output<'static>, mut echo: Input<'static>) {
    loop {
        let r = measure(&mut trig, &mut echo).await;
        SONAR.lock(|s| s.set(Some((r, Instant::now()))));
        Timer::after(PING_INTERVAL).await;
    }
}

/// Turns (command, sonar) into motor speeds, CONTROL_PERIOD at a time.
#[embassy_executor::task]
async fn control_task(mut left: Motor<'static>, mut right: Motor<'static>) {
    let mut blocked = true; // until the sonar has said otherwise
    let mut clear_streak: u8 = 0;
    let mut last_seen: Option<Instant> = None;
    let mut applied: (Speed, Speed) = (0, 0);
    let mut last_report = Instant::now();
    loop {
        Timer::after(CONTROL_PERIOD).await;
        let now = Instant::now();

        // Sonar gate: hysteresis (STOP_MM / GO_MM) plus CLEAR_CONFIRM debounce on the clear side.
        let reading = SONAR.lock(|s| s.get());
        let fresh = reading.filter(|(_, at)| now.saturating_duration_since(*at) <= SENSOR_TIMEOUT);
        let was_blocked = blocked;
        match fresh {
            None => {
                // Silent sensor: block now, and start the streak over when it comes back.
                blocked = true;
                clear_streak = 0;
            }
            // The loop runs faster than the sonar pings; only count each reading once.
            Some((_, at)) if last_seen == Some(at) => {}
            Some((r, at)) => {
                last_seen = Some(at);
                let clear = match r {
                    Ok(mm) if mm < STOP_MM => Some(false),
                    Ok(mm) if mm > GO_MM => Some(true),
                    Ok(_) => None,                        // hysteresis band: no opinion
                    Err(SonarError::OutOfRange) => Some(true), // nothing ahead
                    Err(_) => Some(false),                // faulty: don't go forward blind
                };
                match clear {
                    Some(false) => {
                        blocked = true;
                        clear_streak = 0;
                    }
                    Some(true) => {
                        clear_streak = clear_streak.saturating_add(1);
                        if clear_streak >= CLEAR_CONFIRM {
                            blocked = false;
                        }
                    }
                    None => clear_streak = 0,
                }
            }
        }
        if blocked != was_blocked {
            log::info!("sonar {} ({:?})", if blocked { "BLOCKED" } else { "clear" }, fresh.map(|(r, _)| r));
        }

        let c = command();
        let mut cmd = c.cmd;
        if cmd != Cmd::Stop && now.saturating_duration_since(c.at) > MAX_HOLD {
            log::info!("{:?} held over {} s: auto-stop", cmd, MAX_HOLD.as_secs());
            set_command(Cmd::Stop);
            cmd = Cmd::Stop;
        }
        let effective = if cmd == Cmd::Forward && blocked { Cmd::Stop } else { cmd };
        STATUS.lock(|s| s.set((effective, blocked)));

        let s = c.speed;
        let want: (Speed, Speed) = match effective {
            Cmd::Stop => (0, 0),
            Cmd::Forward => (s, s),
            Cmd::Reverse => (-s, -s),
            Cmd::Left => (-s, s),
            Cmd::Right => (s, -s),
        };
        if want != applied {
            applied = want;
            left.set(want.0 * LEFT_SIGN);
            right.set(want.1 * RIGHT_SIGN);
            log::info!("motors L {} R {}", want.0, want.1);
        }

        if now.saturating_duration_since(last_report) >= Duration::from_secs(1) {
            last_report = now;
            match fresh.map(|(r, _)| r) {
                Some(Ok(mm)) => log::info!("{:?}  {} mm", effective, mm),
                Some(Err(e)) => log::info!("{:?}  {:?}", effective, e),
                None => log::info!("{:?}  sonar silent", effective),
            }
        }
    }
}

// ---------------------------------------------------------------- Bluetooth

#[gatt_server]
struct Server {
    nus: NusService,
}

/// Nordic UART Service: the de-facto serial port over BLE.
#[gatt_service(uuid = "6e400001-b5a3-f393-e0a9-e50e24dcca9e")]
struct NusService {
    /// Phone -> robot
    #[characteristic(uuid = "6e400002-b5a3-f393-e0a9-e50e24dcca9e", write, write_without_response)]
    rx: [u8; 20],
    /// Robot -> phone
    #[characteristic(uuid = "6e400003-b5a3-f393-e0a9-e50e24dcca9e", notify)]
    tx: [u8; 20],
}

const NUS_UUID_LE: [u8; 16] = [0x9e, 0xca, 0xdc, 0x24, 0x0e, 0xe5, 0xa9, 0xe0, 0x93, 0xf3, 0xa3, 0xb5, 0x01, 0x00, 0x40, 0x6e];

/// Fixed 20-byte line buffer (one default-MTU notification) with `fmt::Write`; truncates.
struct Line {
    buf: [u8; 20],
    len: usize,
}

impl Line {
    fn new() -> Self {
        Self { buf: [0; 20], len: 0 }
    }
    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl core::fmt::Write for Line {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let b = s.as_bytes();
        let n = b.len().min(self.buf.len() - self.len);
        self.buf[self.len..self.len + n].copy_from_slice(&b[..n]);
        self.len += n;
        if n < b.len() { Err(core::fmt::Error) } else { Ok(()) }
    }
}

/// Serve GATT requests until the phone goes away. Writes to RX become commands.
async fn gatt_events(conn: &GattConnection<'_, '_, DefaultPacketPool>, rx_handle: u16) {
    loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => {
                log::info!("phone disconnected: {:?}", defmt::Debug2Format(&reason));
                break;
            }
            GattConnectionEvent::Gatt { event } => {
                if let GattEvent::Write(ev) = &event {
                    if ev.handle() == rx_handle {
                        ev.with_data(|_offset, data| handle_rx(data));
                    }
                }
                match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => log::warn!("gatt reply: {:?}", defmt::Debug2Format(&e)),
                }
            }
            _ => {}
        }
    }
}

/// Once a second, tell the phone what we are doing.
async fn telemetry(server: &Server<'_>, conn: &GattConnection<'_, '_, DefaultPacketPool>) {
    loop {
        Timer::after_secs(1).await;
        let (cmd, blocked) = STATUS.lock(|s| s.get());
        let sonar = SONAR.lock(|s| s.get());
        let mut line = Line::new();
        let _ = match sonar {
            Some((Ok(mm), _)) => core::write!(line, "{:?} {}mm", cmd, mm),
            Some((Err(e), _)) => core::write!(line, "{:?} {:?}", cmd, e),
            None => core::write!(line, "{:?} no sonar", cmd),
        };
        if blocked {
            let _ = line.write_str(" !");
        }
        let _ = line.write_str("\n");
        // Fails harmlessly if the app hasn't subscribed to TX.
        let _ = server.nus.tx.notify_raw(conn, line.as_bytes(), false).await;
    }
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(unwrap!(logger_task(driver)));

    // Motors first, so they are known to be coasting before anything else runs.
    let cfg = pwm_config();
    // Motor A (GP0/GP1) is the LEFT wheel, motor B (GP2/GP3) the right (bench check 2026-10-15).
    let left = Motor::new(Pwm::new_output_ab(p.PWM_SLICE0, p.PIN_0, p.PIN_1, cfg.clone()), cfg.clone());
    let right = Motor::new(Pwm::new_output_ab(p.PWM_SLICE1, p.PIN_2, p.PIN_3, cfg.clone()), cfg);
    spawner.spawn(unwrap!(control_task(left, right)));

    let trig = Output::new(p.PIN_16, Level::Low);
    let echo = Input::new(p.PIN_17, Pull::Down);
    spawner.spawn(unwrap!(sonar_task(trig, echo)));

    // Radio bring-up, identical to ble_test.
    let fw = aligned_bytes!("../../firmware/43439A0.bin");
    let clm = aligned_bytes!("../../firmware/43439A0_clm.bin");
    let btfw = aligned_bytes!("../../firmware/43439A0_btfw.bin");
    let nvram = aligned_bytes!("../../firmware/nvram_rp2040.bin");
    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common, pio.sm0, RM2_CLOCK_DIVIDER, pio.irq0, cs, p.PIN_24, p.PIN_29,
        dma::Channel::new(p.DMA_CH0, Irqs), dma::Channel::new(p.DMA_CH1, Irqs),
    );
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, bt_device, mut control, runner) = cyw43::new_with_bluetooth(state, pwr, spi, fw, btfw, nvram).await;
    spawner.spawn(unwrap!(cyw43_task(runner)));
    control.init(clm).await;

    Timer::after_millis(1500).await;
    log::info!("wheely-remote: stop < {} mm, go > {} mm, max hold {} s", STOP_MM, GO_MM, MAX_HOLD.as_secs());

    let controller: ExternalController<_, 10> = ExternalController::new(bt_device);
    let address = Address::random([0xc2, 0x4e, 0x11, 0x05, 0xe4, 0xff]);
    let mut resources: HostResources<DefaultPacketPool, 1, 2> = HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address).build();
    let mut runner = stack.runner();
    let mut peripheral = stack.peripheral();

    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "wheely",
        appearance: &appearance::UNKNOWN,
    }))
    .unwrap();
    let rx_handle = server.nus.rx.handle;

    let ble = async {
        loop {
            if let Err(e) = runner.run().await {
                log::error!("ble runner: {:?}", defmt::Debug2Format(&e));
            }
        }
    };
    let app = async {
        loop {
            let mut adv = [0u8; 31];
            let len = AdStructure::encode_slice(
                &[
                    AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                    AdStructure::CompleteServiceUuids128(&[NUS_UUID_LE]),
                    AdStructure::CompleteLocalName(b"wheely"),
                ],
                &mut adv[..],
            )
            .unwrap();
            log::info!("advertising as wheely");
            let advertiser = peripheral
                .advertise(&Default::default(), Advertisement::ConnectableScannableUndirected { adv_data: &adv[..len], scan_data: &[] })
                .await
                .unwrap();
            let conn = match advertiser.accept().await {
                Ok(c) => c,
                Err(e) => {
                    log::error!("accept: {:?}", defmt::Debug2Format(&e));
                    continue;
                }
            };
            let conn = conn.with_attribute_server(&server).unwrap();
            log::info!("phone connected");
            set_command(Cmd::Stop);
            select(gatt_events(&conn, rx_handle), telemetry(&server, &conn)).await;
            // Whatever the phone was asking for, it isn't asking any more.
            set_command(Cmd::Stop);
        }
    };
    select(ble, app).await;
}
