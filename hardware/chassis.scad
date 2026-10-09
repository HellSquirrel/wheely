// Wheely printed chassis. Parametric; every number worth changing is up here.
//
// Coordinates: X forward, Y left (+) / right (-), Z up. Floor is Z = 0.
// The plate rides at plate_z above the floor, set by the wheel radius and
// the motor's gearbox height, so the motors hang below the plate on ears.
//
// TT motor numbers are from the Adafruit 3777 drawing (docs/parts/tt_motor_diagram.jpg).
// Everything marked MEASURE is a placeholder until measured on the real part.

// ---------- print / fit ----------
clr        = 0.5;    // clearance added to pockets and lips
hole_m3    = 3.4;    // M3 clearance hole
plate_t    = 4;      // plate thickness
rib_t      = 3;      // stiffening rib thickness
rib_h      = 6;      // rib height below the plate
corner_r   = 8;
$fn        = 48;

// ---------- wheel / ride height ----------
wheel_d    = 60;     // measured 2026-10-09
wheel_w    = 30;     // measured 2026-10-09
wheel_r    = wheel_d / 2;

// ---------- TT motor (from drawing) ----------
tt_len         = 70.0;   // overall
tt_gear_len    = 37.0;   // gearbox section length
tt_w           = 22.44;  // gearbox width (vertical when mounted)
tt_thk         = 18.6;   // gearbox thickness along the axle
tt_shaft_d     = 5.4;
tt_shaft_out   = 8.7;    // shaft beyond each gearbox face
tt_hole_sep    = 17.6;   // the two M3 holes, centre to centre, vertical when mounted
tt_hole_x      = 11.25;  // hole pair is this far from the shaft axis, along the motor
tt_hole_from_end = 11.3; // hole pair from the gearbox end face
ear_t          = 3;      // bracket ear thickness (kit acrylic is 3)

// Plate must clear the top of the gearbox: shaft at wheel_r, gearbox half-width above it.
plate_z    = wheel_r + tt_w / 2 + 4;   // 4 mm over the gearbox: room for a caster spacer and for the motor to breathe

// ---------- plate ----------
plate_l    = 205;    // overall; you place it on the bed
plate_w    = 136;
motor_x    = -plate_l / 2 + 40;  // shaft axis X; motors at the rear, cans pointing forward
motor_y    = plate_w / 2 - 6;    // outer ear face Y; wheels sit in wells cut into the plate edge
well_margin = 4;                 // clearance around the wheel in its well

// ---------- caster: 1" swivel plate caster, from photos 2026-10-09 ----------
caster_h        = 41;     // floor to its mounting face. ESTIMATE +-3 mm: confirm with one straight measurement
caster_plate    = 40;     // square mounting plate
caster_hole_sep = [30, 30]; // hole pattern, ESTIMATE +-1.5 mm, absorbed by the slots below
caster_slot     = [4.5, 7];  // slotted holes, long axis pointing at the pattern centre
caster_x        = 58;            // bolts counterbored from the top; fit them before the breadboard goes on
caster_cbore_d  = 9;  caster_cbore_h = 2.5;   // bolt head recess over the slot (M3 or M4 pan head + washer)

// ---------- breadboard (MEASURE) ----------
bb_l = 170; bb_w = 50; bb_h = 10;  // measured 2026-10-09
bb_pos = [plate_l / 2 - bb_l / 2 - 3, 30];   // left side, pushed to the front edge: the sonar case plugs into the front rows and looks out over the edge; USB exits at the rear
lip_h  = 2.5; lip_t = 2;

// ---------- battery holder 4xAA (MEASURE) ----------
bat_l = 70; bat_w = 60; bat_h = 20;  // measured 2026-10-09; switch is recessed in one face (TODO: which)
bat_pos = [5, -33];                 // right side, forward of the wheel wells
strap_slot = [4, 14];               // zip-tie / velcro slot, X by Y
// Switch is recessed in the TOP face (photo 2026-10-09), so nothing on the plate for it. Wires leave the
// edge at the corner beside the switch: notch the lip there. Orient the box so that corner points at the breadboard.
bat_wire_corner = [1, 1];           // [+1/-1 along X, +1/-1 along Y]: the corner that gets the notch
bat_wire_notch  = 12;               // notch width along the lip

// ---------- sonar ----------
// The printed i-Boxlt 'Bread Board Mount' case (49.2 x 23.4 mm, no screw holes) stands the sensor
// on its own pins in the breadboard's front rows. Nothing on the plate for it.

// =====================================================================
module rounded_plate(l, w, t, r) {
    linear_extrude(t)
        offset(r) offset(-r) square([l, w], center = true);
}

// One motor's pair of ears, hanging from the plate. Mirrored for the other side.
module motor_ears(side) {  // side = +1 left, -1 right
    ear_h = plate_z - (wheel_r - tt_hole_sep / 2 - 5);   // reach 5 mm below the lower hole
    inner_y = motor_y - ear_t;                            // outer ear inner face
    for (y = [motor_y - ear_t, motor_y - ear_t - (tt_thk + clr) - ear_t])
        translate([motor_x - tt_hole_x - 12, side * y + (side > 0 ? 0 : -ear_t), plate_z - ear_h])
            difference() {
                union() {
                    cube([24, ear_t, ear_h + 0.01]);
                    // gusset foot: 8 mm tall wedge on the face away from the gearbox
                    g = (y == motor_y - ear_t) ? 1 : -1;   // outer ear: wedge outboard; inner ear: inboard
                    translate([0, (side * g > 0) ? ear_t : -6, ear_h - 8])
                        hull() {
                            cube([24, 6, 0.01]);
                            translate([0, 0, 7.99]) cube([24, 6, 0.01]);
                            translate([0, (side * g > 0) ? 0 : 6 - 0.01, 0]) cube([24, 0.01, 8]);
                        }
                }
                // two M3 holes, one above the other, 17.6 apart, centred on the shaft height
                for (dz = [-tt_hole_sep / 2, tt_hole_sep / 2])
                    translate([12, -1, wheel_r - (plate_z - ear_h) + dz])
                        rotate([-90, 0, 0]) cylinder(d = hole_m3, h = ear_t + 2);
            }
}

// A ghost of the motor so the preview shows where it sits. Not exported.
module motor_ghost(side) {
    %translate([motor_x - (tt_hole_x + tt_hole_from_end), 0, wheel_r])
        translate([0, side * (motor_y - ear_t - tt_thk / 2 - clr / 2), 0])
            rotate([0, 0, 0])
                translate([0, -tt_thk / 2, -tt_w / 2]) cube([tt_len, tt_thk, tt_w]);
    %translate([motor_x, side * (motor_y + 1 + wheel_w / 2), wheel_r]) rotate([90, 0, 0])
        cylinder(d = wheel_d, h = wheel_w, center = true);
}

// A slot pointing from (x, y) towards the pattern centre, so the caster's hole pattern can be off by a bit.
module caster_slot_cut(dx, dy, h) {
    translate([dx * caster_hole_sep[0] / 2, dy * caster_hole_sep[1] / 2, 0])
        rotate([0, 0, atan2(dy * caster_hole_sep[1], dx * caster_hole_sep[0])])
            hull() for (o = [-1, 1]) translate([o * (caster_slot[1] - caster_slot[0]) / 2, 0, 0]) cylinder(d = caster_slot[0], h = h);
}

module caster_mount() {
    spacer = plate_z - caster_h;
    assert(spacer >= 0, "caster is taller than the plate height; raise plate_z or choose a shorter caster");
    translate([caster_x, 0, caster_h])
        difference() {
            translate([-(caster_plate + 4) / 2, -(caster_plate + 4) / 2, 0])
                cube([caster_plate + 4, caster_plate + 4, spacer + 0.01]);
            for (dx = [-1, 1], dy = [-1, 1])
                translate([0, 0, -1]) caster_slot_cut(dx, dy, spacer + plate_t + 2);
        }
}

module lip(size, pos) {   // a low wall around a footprint, with clearance
    translate([pos[0], pos[1], plate_z + plate_t])
        linear_extrude(lip_h)
            difference() {
                offset(lip_t) square([size[0] + clr, size[1] + clr], center = true);
                square([size[0] + clr, size[1] + clr], center = true);
            }
}

module side_rails(size, pos) {   // two low walls along the long sides; short ends stay open
    for (sy = [-1, 1])
        translate([pos[0], pos[1] + sy * ((size[1] + clr) / 2 + lip_t / 2), plate_z + plate_t])
            linear_extrude(lip_h) square([size[0] + clr, lip_t], center = true);
}

rib_y = 22;   // inboard of the motor ears (which start at motor_y - 2*ear_t - tt_thk), clear of the caster block and strap slots
module ribs() {
    for (y = [-rib_y, rib_y])
        translate([-plate_l / 2 + 10, y - rib_t / 2, plate_z - rib_h])
            cube([plate_l - 20, rib_t, rib_h + 0.01]);
}

module plate_cutouts() {
    // wheel wells: notch the plate edge around each wheel
    for (sy = [-1, 1])
        translate([motor_x - (wheel_r + well_margin), sy > 0 ? motor_y - 1 : -(plate_w / 2 + 1), plate_z - 1])
            cube([2 * (wheel_r + well_margin), plate_w / 2 + 1 - (motor_y - 1) + 0.01, plate_t + 2]);
    // caster bolt head recesses from the top (round, big enough to cover the slot)
    translate([caster_x, 0, plate_z + plate_t - caster_cbore_h])
        for (dx = [-1, 1], dy = [-1, 1])
            translate([dx * caster_hole_sep[0] / 2, dy * caster_hole_sep[1] / 2, 0])
                cylinder(d = caster_cbore_d, h = caster_cbore_h + 1);
    // caster slots through the plate
    translate([caster_x, 0, plate_z - 1])
        for (dx = [-1, 1], dy = [-1, 1]) caster_slot_cut(dx, dy, plate_t + 2);
    // battery strap slots, one each side of the holder
    for (s = [-1, 1])
        translate([bat_pos[0] + s * (bat_l / 2 + 4), bat_pos[1], plate_z - 1])
            linear_extrude(plate_t + 2) square(strap_slot, center = true);
    // cable pass-through between the battery bay and the breadboard
    translate([bat_pos[0] - bat_l / 2 + 10, (bat_pos[1] + bb_pos[1]) / 2, plate_z - 1])
        linear_extrude(plate_t + 2) square([12, 6], center = true);
}

module chassis() {
    difference() {
        union() {
            translate([0, 0, plate_z]) rounded_plate(plate_l, plate_w, plate_t, corner_r);
            ribs();
            for (s = [-1, 1]) motor_ears(s);
            caster_mount();
            side_rails([bb_l, bb_w], bb_pos);
            difference() {
                lip([bat_l, bat_w], bat_pos);
                // wire notch: cut the lip at the chosen corner, on the X-facing end
                translate([bat_pos[0] + bat_wire_corner[0] * (bat_l / 2 + 2), bat_pos[1] + bat_wire_corner[1] * (bat_w / 2 - bat_wire_notch / 2 - 2), plate_z + plate_t - 1])
                    linear_extrude(lip_h + 2) square([2 * lip_t + clr + 4, bat_wire_notch], center = true);
            }
        }
        plate_cutouts();
    }
}

chassis();
for (s = [-1, 1]) motor_ghost(s);
// ghost: breadboard and the sonar case standing on its front rows
%translate([bb_pos[0], bb_pos[1], plate_z + plate_t]) linear_extrude(bb_h) square([bb_l, bb_w], center = true);
%translate([bb_pos[0] + bb_l / 2 - 12, bb_pos[1], plate_z + plate_t + bb_h]) linear_extrude(23.4) square([23.4, 49.2], center = true);
%translate([bat_pos[0], bat_pos[1], plate_z + plate_t]) linear_extrude(bat_h) square([bat_l, bat_w], center = true);
%translate([caster_x, 0, 0]) { translate([0, 0, caster_h - 1]) linear_extrude(1) square(caster_plate, center = true); translate([6, 0, 12.5]) rotate([90, 0, 0]) cylinder(d = 25, h = 10, center = true); }
