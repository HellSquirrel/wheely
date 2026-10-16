# wheely

A two-wheel-drive robot built to learn embedded Rust. Raspberry Pi Pico 2 W (RP2350 + CYW43439
radio), Embassy async firmware, a 3D-printed chassis, a phone/browser remote over Bluetooth LE,
an obstacle-avoiding cruise mode, and Wi-Fi telemetry for logging and mapping.

Prototype complete 2026-10-16. This file is the handoff: hardware, wiring, firmware, tools,
workflows, tunables and the gotchas that cost real time.

## Hardware

| Part | Notes |
|---|---|
| Raspberry Pi Pico 2 W | RP2350 (ARM Cortex-M33 selected), CYW43439 Wi-Fi (2.4 GHz only) + Bluetooth LE, 4 MB flash |
| Adafruit DRV8833 breakout | Dual H-bridge, 1.2 A/ch. SLP must be high (internal 500 kΩ pull-down) |
| 2× TT gear motors + 60×30 mm wheels | From a DollaTek 2WD kit. Motor A (GP0/GP1) is the LEFT wheel, B (GP2/GP3) the right |
| RCWL-1601 ultrasonic sensor | HC-SR04 pin-compatible, 3.3 V native: no level shifting. Transducer centres 22.5 mm |
| Pololu D45V5F3 | 3.3 V step-down (input 4–65 V) feeding VSYS through a 1N5819 Schottky |
| 4×AA holder (6 V alkaline) | Its slide switch died; bypassed. Power now switched by a latching push switch in the regulator's VIN line only |
| 2× 220 µF 10 V | In parallel across the battery leads |
| 1" swivel plate caster | Holes 30 × 23.5 mm (asymmetric), Ø5, 35 mm tall |
| Full-size breadboard 170×50×10 | Rear edge of the plate. A Perma-Proto rebuild is a later option |
| Raspberry Pi Debug Probe | Owned, not used: flashing is BOOTSEL + picotool, logs over USB CDC |
| FNIRSI 2C53T | Scope/meter. Leads must be pushed in hard; HOLD freezes the display; red lead in the V/Ω jack |

Why the regulator: 4×AA alkaline is 6.0–6.4 V, above the Pico's VSYS maximum of 5.5 V. The 3.3 V
regulator output into VSYS (allowed range 1.8–5.5 V) never drops out over the pack's life. USB and
battery each reach VSYS through their own diode, so plugging USB in with the battery on is fine.
The DRV8833's VM is on the battery directly; with the Pico off, SLP floats low and the driver sleeps.

## Wiring

Full diagram: `docs/wheely-wiring.pdf`. Pinout: `docs/pico-2-w-pinout-official.pdf`.

| Pico | Signal |
|---|---|
| GP0 / GP1 | DRV8833 AIN1 / AIN2, motor A (left), PWM slice 0 |
| GP2 / GP3 | DRV8833 BIN1 / BIN2, motor B (right), PWM slice 1 |
| GP16 | Sonar Trig |
| GP17 | Sonar Echo (pull-down) |
| 3V3 (pin 36) | Breadboard red rail: sonar VCC, DRV8833 SLP |
| VSYS (pin 39) | From the regulator via the Schottky |
| GND | Common ground for Pico, driver, regulator, battery and sensor |

Radio pins (GP23/24/25/29) are internal to the Pico 2 W.

## Chassis (`hardware/`)

OpenSCAD, parametric, printed in PLA. Install OpenSCAD with `brew install --cask openscad@snapshot`
(the plain cask is disabled). The plate prints top-face-down without supports, so nothing may stand
on its top face; everything that does is a separate bolt-on part.

| File | What |
|---|---|
| `chassis.scad` → `wheely_chassis_v15.stl` | The plate: motors rear, caster front, breadboard rear edge, battery centre-right, strap slots |
| `wheely_motor_mount_v11.stl` | Single-sided TT motor mount (5 mm inboard ear, bolts from the wheel side, nuts inboard). Passed |
| `wheely_caster_mount_v11.stl` | Caster spacer/mount, slotted for the 30 × 23.5 pattern |
| `wheely_sonar_bracket_v16.stl` | Upright bracket for the hinged sonar case, on the plate's two front holes |
| `third_party/hc_sr04_hinged_case_22p5mm.stl` | MakerWorld model 1913595 with the openings moved 1.75 mm inboard each for this sensor |
| `assembly.scad` | Coloured preview of everything in place |
| `*_test.scad` | Coupons printed before committing to a full plate |

Screws: 6× M3×30 (motors), 4× M3×25 (caster), 2× M3×10 (case ears), nuts and washers.

TT motor facts that broke the first mounts: the M3 holes are on the motor-can side of the shaft,
20 mm from it and 17.6 mm apart; Ø4 pegs 2–3 mm tall sit on both broad faces ~10 mm from the
shaft; the gearbox is wider than the published drawings. Anything in the gearbox's face planes
collides with something.

Keep versioned STL filenames. Same-named downloads got mixed up more than once.

## Firmware (`src/`)

Rust, `thumbv8m.main-none-eabihf`, Embassy. All embassy crates are pinned in `[patch.crates-io]` to
git revision `3cd51e6` because the released `cyw43 0.7` and `trouble-host 0.8` disagree on the
Bluetooth HCI crate version. The version numbers match crates.io, so nothing else changed.

Library modules:

- `motor.rs`: DRV8833 pair on one PWM slice, 20 kHz, speed in permille. SLOW decay (one input
  held high, PWM the other). Fast decay would not start a TT motor below ~100 % duty.
- `sonar.rs`: 10 µs trigger, echo timing with timeouts, result in mm or `NoEcho`/`OutOfRange`/`Busy`.
- `cruise.rs`: obstacle-avoidance state machine (forward, back 300 ms, spin, settle 400 ms).
- `text.rs`: fixed-size `fmt::Write` buffer.

Binaries (`cargo run --release --bin <name>`):

| Binary | Purpose |
|---|---|
| `remote` | **The robot.** BLE remote, cruise mode, Wi-Fi telemetry, 1 Hz status to the phone, `boot` command |
| `drive` | Milestone 3: drive forward, stop under 20 cm, resume over 30 cm |
| `sonar` | Ranging test over USB log |
| `sonar_diag` | Finds swapped Trig/Echo |
| `motor` | PWM ramp test |
| `probe` | ADC voltmeter on GP26. Saturates at 3299 mV, so a 3.3 V node must be read through a loaded diode |
| `bisect` | Phase bisect helper |
| `ble_test` | Advertise + log received bytes |

### Remote protocol

Nordic UART Service (`6e400001-…`), device name `wheely`, one connection at a time.

- Bluefruit Connect control pad packets `!B<button><state><~sum>`: 5 up, 6 down, 7 left, 8 right,
  1–4 speed presets (300/500/750/1000 permille). Hold to move, release to stop.
- Plain text: `f b l r s` directions and stop, `1`–`4` speed, `c` cruise, `boot` reboot into BOOTSEL.
- Status notify once a second: `Forward 523mm`, `!` appended while the sonar blocks forward,
  `Cruise/turning 180mm` while cruising.

Safety, in order: disconnect stops everything; a held direction stops after 5 s (press again);
the sonar refuses forward under 20 cm (one reading) and allows it again after three consecutive
readings over 30 cm; a faulty or silent sonar cancels cruise.

Tunables are the constants at the top of `src/bin/remote.rs` and `src/cruise.rs`.

### Wi-Fi telemetry

Credentials in `wifi.secrets` (gitignored, template in `wifi.secrets.example`), read by `build.rs`.
Quotes around values are tolerated. Without the file the firmware builds with Wi-Fi off.
The robot joins by DHCP and broadcasts a JSON line every 100 ms on UDP port 4210:

```json
{"seq":937,"t":46947,"mm":292,"sonar":"ok","blocked":true,"cmd":"Stop","eff":"Stop","cruise":"off","speed":500,"l":0,"r":0,"ble":false}
```

`t` is uptime in ms, `l`/`r` the wheel speeds in permille, `cruise` the phase. On a failed join the
firmware scans and logs every visible network with its channel, and rotates the authentication
mode (mixed, WPA2, WPA3). Set `TELEMETRY_HOST` in `wifi.secrets` to unicast to one laptop if the
network blocks broadcasts.

## Tools

- `web/`: the browser remote (Web Bluetooth PWA). Serve it locally and open in Chrome:
  `python3 -m http.server 8077 --bind 127.0.0.1 --directory web` → http://127.0.0.1:8077/.
  Arrow keys or WASD, space stops, 1–4 speed, C cruise. Works in Chrome/Edge on Android, Windows,
  macOS, ChromeOS; not Safari, not Firefox. A phone needs an HTTPS host.
- `tools/telemetry.py`: logs the UDP stream to CSV. Standard library only.
- `tools/map.py`: dead-reckoned live map (path + sonar hits). Needs matplotlib (`.venv` has it:
  `.venv/bin/python tools/map.py`). Calibrate `V_FULL_MM_S` and `TRACK_MM` at the top first.
  Without encoders the pose drifts; the kit's encoder discs are the upgrade path.

## Workflows

Build and flash (board in BOOTSEL):

```
cargo run --release --bin remote
```

Getting into BOOTSEL: send `boot` over BLE (UART screen or the web app's Dev button). The manual
way is battery off, unplug USB, hold BOOTSEL, plug USB. With the battery on, unplugging USB does
not reset the Pico, so the button alone does nothing.

Logs: `screen /dev/cu.usbmodem* 115200` (USB CDC, 1 KB buffered before the host connects).

## Gotchas, in the order they bit

1. Fast-decay PWM looked exactly like a wiring fault. Slow decay fixed it.
2. Trig/Echo swapped after rewiring: `sonar_diag` finds it.
3. The breadboard's red rail had no jumper from 3V3 (pin 36).
4. First chassis print: ear snapped, motor holes on the wrong side of the shaft, blind hole, pegs.
5. The Pico ADC cannot tell 3.3 V from 6 V.
6. "Dead motors" were a dead battery-holder switch, found with the meter after learning the meter.
7. Wheels-in-the-air turn direction was misread once; trust floor tests.
8. A close, angled object loses echoes; one "nothing in range" reading must not unblock forward.
9. `use defmt::*` shadows `write!`; use `core::write!` for `fmt::Write` targets.
10. Shell-style quotes in `wifi.secrets` became part of the network name.
11. BLE devices never appear in the phone's Bluetooth settings, only inside the app.

## Next stage candidates

Wheel encoders (RP2350 PIO) for real odometry and a usable map; smarter cruise (wall following);
a Perma-Proto rebuild of the breadboard; the ESP32-S3 handheld controller; an LLM on the laptop
driving the robot over Wi-Fi.
