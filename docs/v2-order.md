# wheely v2: order checklist (2026-10-16)

Option B power (1S LiPo + LiPo Amigo Pro), breadboard first, soldered build after.
Not needed again: Perma-Proto 3-pack, capacitor kit, 1N5819 diodes, M3 hardware, jumper wires, meter, soldering kit.
Boards are not screwed down: the printed chassis holds the Perma-Proto in a tray with edge rails and the charger in a pocket; the Pico sits in its header socket. M3 only, for the mechanical parts.

## The Pi Hut / Pimoroni

- [ ] 1 × Raspberry Pi Pico 2 WH (headers pre-soldered)
- [ ] 1 × Pimoroni LiPo Amigo Pro (Pi Hut lists the Pro as unavailable; Pimoroni's shop has it, SKU PIM612)
- [ ] 1 × PKCell 2000 mAh 3.7 V LiPo, JST-PH plug, built-in protection (60 × 38 × 8 mm, £10, thepihut.com/products/2000mah-3-7v-lipo-battery). Check the red wire against the + mark on the Amigo Pro before plugging in
- [ ] 1 × Adafruit DRV8833 DC motor driver breakout
- [ ] 1 × RCWL-1601 ultrasonic sensor (3.3 V, HC-SR04 footprint)
- [ ] 1 × Full-size breadboard, 830 tie points
- [ ] 1 × 40-pin female header 2.54 mm (cut into 2 × 20 for the Pico socket)
- [ ] 2 × 40-pin male header strip 2.54 mm (driver breakout, Amigo Pro switch pads)

## Amazon UK

- [ ] 1 × 2WD TT chassis kit (2 TT motors with wires, 2 wheels 65 mm, caster)
- [ ] 1 × 16 mm metal momentary push button with LED ring, 3 V LED version (12 V versions stay dark on a 3.7 V cell)
- [ ] 2 × 12 mm panel-mount momentary push button (reset; second one for the optional BOOTSEL)
- [ ] 1 × Panel-mount USB extension, USB-C female (panel) to micro-USB male, 20–30 cm
- [ ] 1 × JST-XH 2.54 mm connector kit: through-hole board headers + pre-crimped leads, 2/3/4-pin
- [ ] 1 × JST-PH 2.0 mm 2-pin pigtail pack (male + female)
- [ ] 1 × Female-female Dupont leads, 10 cm
- [ ] 1 × Silicone wire 22 AWG, red + black, 2 m each
- [ ] 1 × Silicone wire 26 AWG, five colours
- [ ] 1 × Heat-shrink tubing assortment
- [ ] 2 × 470 µF 10 V electrolytic capacitor (skip if the capacitor kit has 470 µF)
- [ ] 1 × Small zip ties

## Decide later, after the breadboard tests

- [ ] 6 V step-up regulator rated 2 A or more (e.g. Pololu U3V40F6, check the rating): only if the motors are weak at cell voltage
- [ ] N20 6 V 150:1 gear motors with magnetic encoders × 2, matching 60 mm wheels and brackets (Pololu): the odometry upgrade for the map; changes the chassis
- [ ] 0.96" SSD1306 OLED, I²C: status display
- [ ] NeoPixel stick, 8 × WS2812: status light

## Breadboard tests before soldering

1. Motors on carpet at cell voltage (3.3–4.2 V): decides the step-up.
2. Pico from the cell through the Schottky, USB plugged and unplugged: no reboot.
3. Power button through the Amigo Pro SW pads: one press on, one press off, LED follows.
4. Motor stall with both wheels held: the Amigo Pro's protector (XB6096I2S) is likely to trip near 1 A, so plan on feeding the driver's VM from the VBAT pin (raw battery) and let the Pico's 3V3 on SLP put the driver to sleep when the robot is off, as in v1. The cell's own protection covers under-voltage on that path.
5. Reset via RUN; flash via `boot` over Bluetooth with the panel USB.
