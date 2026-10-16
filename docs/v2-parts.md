# wheely v2: parts list

v2 is a second, soldered robot with a tidier look. v1 stays as it is. Goals: power switch on the
outside, reset and flashing without opening the robot, rechargeable, status lights.

Already owned, no need to rebuy: Adafruit Perma-Proto half-size (3-pack), capacitor kit (220 µF),
1N5819 Schottky diodes, M3 screws/nuts/brass standoffs, jumper wire, Debug Probe, meter, soldering kit.

## Core

| Qty | Part | Notes |
|---|---|---|
| 1 | Raspberry Pi Pico 2 WH (headers pre-soldered) | Plugs into the breadboard for testing, later into female headers on the Perma-Proto, so it stays swappable. Pi Hut / Pimoroni |
| 2 | 20-pin female header strips (2.54 mm) | Pico socket on the Perma-Proto |
| 1 | Adafruit DRV8833 breakout | Same driver as v1: works down to 2.7 V, which matters for the LiPo option |
| 1 | RCWL-1601 ultrasonic sensor | 3.3 V native, same as v1. Print the modified hinged case again |
| 2 | TT gear motor, 1:48, with wires | Cheapest as another 2WD kit (motors + wheels + caster). See the encoder option below |
| 2 | 65 mm wheels for TT motors | In the kit |
| 1 | 1" swivel caster | In the kit, or same as v1 |

## Power: Option B chosen (2026-10-16)

**Option B, recommended: rechargeable 1S LiPo, USB-C charging, no 3.3 V regulator.**
VSYS accepts 1.8–5.5 V, so a single cell (3.0–4.2 V) feeds the Pico directly through a Schottky.
The motors run from the same cell: at 3.7 V they match what v1 does at speed 500 on a 6 V pack.

| Qty | Part | Notes |
|---|---|---|
| 1 | Pimoroni LiPo Amigo Pro | USB-C charge (200 mA, 500 mA with the FAST CHARGE bridge, protected cell only), battery protection, on/off button with LED, SW pads for an external momentary power button, JST-PH in and out |
| 1 | Pimoroni LiPo 2000 mAh (JST-PH, protected) | ~2 h of driving. 1000 mAh if space is tight |
| 1 | 16 mm metal momentary push button with LED ring | Outside. Wires to the Amigo Pro SW pads: one press on, one press off. LED shows power |
| 1 | 470 µF 10 V capacitor | Across VM at the driver. Kit may have it |
| (1) | 6 V step-up regulator, 2 A or more, e.g. Pololu U3V40F6 (check the rating) | Only if the motors turn out weak on carpet at battery voltage |

**Option A, not chosen, kept for reference: 4×AA + regulator, as v1.**

| Qty | Part | Notes |
|---|---|---|
| 1 | 4×AA holder with leads, no switch | v1's switch died |
| 1 | Pololu D45V5F3 3.3 V step-down | As v1 |
| 1 | Panel-mount rocker switch rated 6 A, KCD1 style (15×21 mm cutout) | In the battery lead so it switches everything. Must carry motor current, hence the rating |
| 4 | AA NiMH + charger | Optional. 4.8 V instead of 6 V, rechargeable |

## Outside access: switch, reset, flashing

| Qty | Part | Notes |
|---|---|---|
| 1 | Panel-mount USB extension: USB-C female (panel) to micro-USB male, 20–30 cm | Brings the Pico's USB port to the outside. Flash with `boot` over Bluetooth, then plug in |
| 1 | 12 mm panel-mount momentary push button | RESET: wired between RUN (pin 30) and GND. Raspberry Pi says not to use 3V3_EN for this |
| (1) | 12 mm panel-mount momentary push button | BOOTSEL, optional: soldered to the BOOTSEL test pad on the Pico's underside (TP6 on the original Pico; confirm the pad on the Pico 2 W datasheet). Then hold BOOTSEL, tap RESET, release = flash mode, no software needed |

## Connectors, wire, mounting

| Qty | Part | Notes |
|---|---|---|
| 1 | JST-PH 2.0 mm pre-crimped cable kit with board headers (2-, 3-, 4-pin) | Motors (2-pin) and sonar (4-pin) unplug for service. No crimping tool needed |
| 1 | Silicone wire, 22 AWG red + black, 2 m each | Battery and motor power |
| 1 | Silicone wire, 26 AWG, 5 colours | Signals |
| 1 | Heat-shrink assortment | |
| 1 | M2 + M2.5 nylon standoff and screw kit | Pico holes take M2; Perma-Proto holes take M2.5. M3 you have |
| 1 | Small zip ties / cable clips | |

## Test rig: breadboard v2 before soldering

v1 keeps its breadboard, so v2 needs its own. Everything above gets wired on it first; the Perma-Proto
layout is copied from the working breadboard.

| Qty | Part | Notes |
|---|---|---|
| 1 | Full-size breadboard (830 tie points) | Same as v1's. Half-size gets cramped with the driver, the Pico and the charger |
| 1 | Jumper wire kit (M/M, M/F) | The v1 pack may be enough; a second one is cheap |
| 2 | JST-PH 2-pin pigtails, male-end bare or Dupont | To get the Amigo Pro's battery/device connectors onto the breadboard. Usually in the JST kit |
| 1 | 2.54 mm male header strip | For the DRV8833 breakout (loose headers, as v1) and the Amigo Pro's SW pads |

Breadboard tests that decide the final build: motor strength on carpet at cell voltage (decides the
6 V step-up), the Pico running from the cell through the Schottky with USB plugged and unplugged,
power button on/off through the Amigo Pro's SW pads, and reset via RUN.

## Looks and the next milestone (optional)

| Qty | Part | Notes |
|---|---|---|
| 1 | 0.96" or 1.3" OLED, SSD1306, I²C | IP address, distance, mode on the robot. Two GPIOs |
| 1 | NeoPixel stick (8× WS2812) or ring | Mode and distance as light. One GPIO, PIO driver exists in embassy |
| 2 | N20 micro metal gear motor, 6 V, 150:1, with magnetic encoder, e.g. Pololu HPCB with encoder, plus Pololu 60×8 mm wheels and brackets | Replaces the TT motors. Encoders give real odometry, which is what the map needs. Needs a new chassis; motor pins for the encoders: four GPIOs |

Sources checked 2026-10-16: Pimoroni LiPo Amigo product page; Raspberry Pi "How to add a reset button
to your Raspberry Pi Pico"; pico2w.pinout.xyz for the RUN pin.
