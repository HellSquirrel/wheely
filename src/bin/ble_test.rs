//! Bluetooth smoke test: bring up the CYW43439 with its Bluetooth firmware, advertise as "wheely"
//! with the Nordic UART Service, and log whatever a phone writes to the RX characteristic.
//! Logs go out over USB serial as usual. Pair with e.g. Adafruit's "Bluefruit Connect" app (UART mode)
//! or any BLE terminal app that speaks the Nordic UART Service.
//!
//! Flash: BOOTSEL, then `cargo run --release --bin ble_test`.

#![no_std]
#![no_main]

use cyw43::aligned_bytes;
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::{join::join, select::select};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIO0, USB};
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_rp::{bind_interrupts, dma};
use embassy_time::Timer;
use panic_probe as _;
use static_cell::StaticCell;
use trouble_host::prelude::*;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
});

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

// Nordic UART Service: the de-facto "serial port over BLE". Phone apps know it.
#[gatt_server]
struct Server {
    nus: NusService,
}

#[gatt_service(uuid = "6e400001-b5a3-f393-e0a9-e50e24dcca9e")]
struct NusService {
    /// Phone -> robot
    #[characteristic(uuid = "6e400002-b5a3-f393-e0a9-e50e24dcca9e", write, write_without_response)]
    rx: [u8; 20],
    /// Robot -> phone
    #[characteristic(uuid = "6e400003-b5a3-f393-e0a9-e50e24dcca9e", notify)]
    tx: [u8; 20],
}

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 2;
const NUS_UUID: [u8; 16] = [0x9e, 0xca, 0xdc, 0x24, 0x0e, 0xe5, 0xa9, 0xe0, 0x93, 0xf3, 0xa3, 0xb5, 0x01, 0x00, 0x40, 0x6e]; // little-endian

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(unwrap!(logger_task(driver)));

    // Radio firmware, baked into the image (about 240 KB of the 4 MB flash).
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
    log::info!("wheely-ble: radio up, starting Bluetooth");

    let controller: ExternalController<_, 10> = ExternalController::new(bt_device);
    let address = Address::random([0xc2, 0x4e, 0x11, 0x05, 0xe4, 0xff]);
    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> = HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address).build();
    let mut runner = stack.runner();
    let mut peripheral = stack.peripheral();

    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "wheely",
        appearance: &appearance::UNKNOWN,
    }))
    .unwrap();

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
                    AdStructure::CompleteServiceUuids128(&[NUS_UUID]),
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
                Err(e) => { log::error!("accept: {:?}", defmt::Debug2Format(&e)); continue; }
            };
            let conn = conn.with_attribute_server(&server).unwrap();
            log::info!("phone connected");
            let rx = server.nus.rx;
            let reason = loop {
                match conn.next().await {
                    GattConnectionEvent::Disconnected { reason } => break reason,
                    GattConnectionEvent::Gatt { event } => {
                        if let GattEvent::Write(ev) = &event {
                            if ev.handle() == rx.handle {
                                ev.with_data(|_off, data| {
                                    match core::str::from_utf8(data) {
                                        Ok(s) => log::info!("rx {} bytes: {:?}", data.len(), s),
                                        Err(_) => log::info!("rx {} bytes: {:?}", data.len(), data),
                                    }
                                });
                            }
                        }
                        match event.accept() {
                            Ok(reply) => reply.send().await,
                            Err(e) => log::warn!("gatt reply: {:?}", defmt::Debug2Format(&e)),
                        }
                    }
                    _ => {}
                }
            };
            log::info!("phone disconnected: {:?}", defmt::Debug2Format(&reason));
        }
    };
    let _ = select(ble, join(app, async { loop { Timer::after_secs(60).await; } })).await;
}
