// Wheely printed chassis. Parametric; every number worth changing is up here.
//
// Coordinates: X forward, Y left (+) / right (-), Z up. Floor is Z = 0.
// The plate rides at plate_z above the floor, set by the wheel radius and
// the motor's gearbox height, so the motors hang below the plate on ears.
//
// TT motor numbers are from the Adafruit 3777 drawing (docs/parts/tt_motor_diagram.jpg).
// Everything marked MEASURE is a placeholder until measured on the real part.

// ---------- print / fit ----------
lips       = false;  // raised lips on the top face. true = needs supports under the whole plate; see chassis()
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
tt_thk         = 18.8;   // gearbox thickness along the axle (SparkFun DG01D drawing says 18.8, Adafruit 18.6)
tt_shaft_d     = 5.4;
tt_shaft_out   = 8.7;    // shaft beyond each gearbox face
tt_hole_sep    = 17.6;   // the two M3 holes, centre to centre, vertical when mounted
// Along the motor, from the shaft towards the motor can (docs/parts/tt_motor_dg01d_sparkfun.jpg, which
// is clearer than the Adafruit drawing): a O4 locating peg on each broad face at ~11.25 mm, the two M3
// holes at ~20.6 mm (Adafruit says 22.5), the gearbox/can junction at ~36.5 mm. Free end 14 mm the other way.
// The ears sit from 15 to 35 mm, clear of the pegs, and the bolt holes are SLOTS to absorb the 2 mm disagreement.
tt_end_to_shaft = 14.0;
tt_peg_x       = 11.25;  // peg centre from the shaft, O4, ~2 mm tall, both faces
tt_hole_x      = 20;     // bolt hole centre from the shaft, MEASURED on the kit motor 2026-10-09
tt_hole_slot   = 1;      // slot length along the motor: measured position, so just a touch of play
tt_hole_to_can = 15.0;   // hole pair to the gearbox/can junction (approx)
ear_t          = 3;      // bracket ear thickness (kit acrylic is 3)

// Plate must clear the top of the gearbox: shaft at wheel_r, gearbox half-width above it.
plate_z    = wheel_r + tt_w / 2 + 8;   // 8 mm over the gearbox: a 7.5 mm saddle block ties the motor ears together

// ---------- plate ----------
plate_l    = 205;    // overall; you place it on the bed
plate_w    = 136;
motor_x    = -plate_l / 2 + 40;  // shaft axis X; motors at the rear. Ears are in FRONT of the shaft, cans point forward under the plate
motor_y    = plate_w / 2 - 6;    // outer ear face Y; wheels sit in wells cut into the plate edge
well_margin = 4;                 // clearance around the wheel in its well

// ---------- caster: 1" swivel plate caster, from photos 2026-10-09 ----------
caster_h        = 35;     // floor to its mounting face, measured 2026-10-09 (approx)
caster_plate    = 40;     // square mounting plate
caster_hole_sep = [30, 23.5]; // hole pattern, measured: 30 one way, 23-24 the other (asymmetric). Rotate the caster to match
caster_slot     = [4.5, 7];  // slotted holes (caster holes are ~5 mm; M3 or M4 bolts), long axis pointing at the pattern centre
caster_x        = 58;            // bolts counterbored from the top; fit them before the breadboard goes on
caster_cbore_d  = 9;  caster_cbore_h = 2.5;   // bolt head recess over the slot (M3 or M4 pan head + washer)

// ---------- breadboard (MEASURE) ----------
bb_l = 170; bb_w = 50; bb_h = 10;  // measured 2026-10-09
bb_pos = [-plate_l / 2 + bb_l / 2 + 2, 30];  // left side, pushed to the REAR edge (USB cable exits backwards); the sonar cradle takes the front
lip_h  = 2.5; lip_t = 2;

// ---------- battery holder 4xAA (MEASURE) ----------
bat_l = 70; bat_w = 60; bat_h = 20;  // measured 2026-10-09; switch is recessed in one face (TODO: which)
bat_pos = [5, -33];                 // right side, forward of the wheel wells
strap_slot = [4, 14];               // zip-tie / velcro slot, X by Y
// Switch is recessed in the TOP face (photo 2026-10-09), so nothing on the plate for it. Wires leave the
// edge at the corner beside the switch: notch the lip there. Orient the box so that corner points at the breadboard.
bat_wire_corner = [1, 1];           // [+1/-1 along X, +1/-1 along Y]: the corner that gets the notch
bat_wire_notch  = 12;               // notch width along the lip

// ---------- sonar cradle ----------
// The printed i-Boxlt case (49.2 wide, 23.4 tall, no screw holes) can't stand in breadboard rows
// because the sensor pins carry jumper connectors. So: two towers at the front edge support the
// case's ends, the connectors hang in the open channel between them, wires leave to the rear.
sonar_case_w = 49.2;   // from the 3MF
sonar_case_d = 19;     // assembled front-to-back depth, confirmed 2026-10-09
sonar_y      = 0;      // cradle centre across the plate
sonar_foot   = 17;     // width of each tower under the case ends; channel between = 15 mm, the pin opening is 10
sonar_ped_h  = 15;     // tower height: connectors reach 10 mm below the case, plus bend room
sonar_wall_h = 12;     // end walls rise this much above the towers to locate the case
sonar_wall_t = 3;
sonar_lip    = 2;      // front/rear lips, 5 mm tall, keep it from sliding
sonar_x_front = plate_l / 2 - 3;   // front face of the cradle

// =====================================================================
module rounded_plate(l, w, t, r) {
    linear_extrude(t)
        offset(r) offset(-r) square([l, w], center = true);
}

// One motor mount, mirrored for the other side. Two 3 mm ears either side of the gearbox,
// joined at the top by a solid saddle that fills the gap between gearbox and plate, so the
// ears hang from a 24 x 25 mm block instead of a 3 mm edge. Gussets run along X (front and
// back of each ear) where there is always plate above them.
ear_len   = 18;      // ears span 15..33 mm from the shaft: pegs (to ~12) behind, can (from ~36) in front
ear_h     = plate_z - (wheel_r - tt_hole_sep / 2 - 5);   // reach 5 mm below the lower hole
saddle_h  = plate_z - (wheel_r + tt_w / 2) - 0.5;         // down to 0.5 mm above the gearbox
gusset_l  = 8;                                              // how far the gussets reach along X
pocket_w  = tt_thk + clr;                                   // gearbox pocket between the ears
mount_w   = 2 * ear_t + pocket_w;                           // overall width across both ears

module motor_mount(side) {   // side = +1 left, -1 right
    x0 = motor_x + 15;                                      // ear block rear edge, 15 mm in front of the shaft (pegs end at ~13.3)
    y_out = motor_y;                                        // outer face of the outer ear
    // build in +Y then mirror for the right side
    mirror([0, side < 0 ? 1 : 0, 0])
    difference() {
        union() {
            // two ears
            for (y = [y_out - ear_t, y_out - mount_w])
                translate([x0, y, plate_z - ear_h]) cube([ear_len, ear_t, ear_h + 0.01]);
            // saddle: solid across both ears, from the plate down to just above the gearbox
            translate([x0, y_out - mount_w, plate_z - saddle_h]) cube([ear_len, mount_w, saddle_h + 0.01]);
            // NO gussets in the ear planes. Behind the ears sit the motor's O4 pegs (8..12 mm from the
            // shaft); in front of them the motor can, which is slightly wider than the gearbox. Anything
            // in those planes beyond the ears collides. The 7.5 mm saddle is the structural tie.
            // inner ear: two fins on its inboard face (away from the pocket), at its front and rear edges,
            // clear of the bolt holes so the nuts seat flat
            for (xf = [x0, x0 + ear_len - 3])
                translate([xf, y_out - mount_w, plate_z - ear_h + 6])
                    hull() {
                        cube([3, 0.01, ear_h - 6]);
                        translate([0, -6, ear_h - 6 - 0.01]) cube([3, 6, 0.01]);
                    }
        }
        // two M3 SLOTS through both ears, one above the other, centred on the shaft height,
        // elongated along the motor so a 2 mm error in the hole position doesn't matter
        for (dz = [-tt_hole_sep / 2, tt_hole_sep / 2])
            hull() for (dx = [-tt_hole_slot / 2, tt_hole_slot / 2])
                translate([motor_x + tt_hole_x + dx, y_out - mount_w - 1, wheel_r + dz])
                    rotate([-90, 0, 0]) cylinder(d = hole_m3, h = mount_w + 2);
        // lead-in chamfer on the pocket at the ears' open (lower) end
        for (y = [y_out - ear_t, y_out - mount_w + ear_t])
            translate([x0 - 1, y, plate_z - ear_h - 0.01])
                rotate([45, 0, 0]) cube([ear_len + 2, 1.4, 1.4], center = false);
        // keep the pocket open: the saddle must not reach below the plate between the ears
        // except as the saddle itself (nothing to cut); the gearbox clears it by 0.5 mm.
    }
}

// A ghost of the motor so the preview shows where it sits. Not exported.
module motor_ghost(side) {
    %translate([motor_x - tt_end_to_shaft, 0, wheel_r])
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
module sonar_cradle() {
    pocket_d = sonar_case_d + clr;                 // X
    pocket_w = sonar_case_w + clr;                 // Y
    x1 = sonar_x_front; x0 = x1 - pocket_d - 2 * sonar_lip;
    z0 = plate_z + plate_t;
    translate([0, sonar_y, 0]) union() {
        // two towers under the case ends, full cradle depth, with the end wall rising above
        for (sy = [-1, 1]) {
            translate([x0, sy > 0 ? pocket_w / 2 - sonar_foot : -pocket_w / 2, z0 - 0.01])
                cube([pocket_d + 2 * sonar_lip, sonar_foot, sonar_ped_h + 0.01]);
            translate([x0, sy > 0 ? pocket_w / 2 : -pocket_w / 2 - sonar_wall_t, z0 - 0.01])
                cube([pocket_d + 2 * sonar_lip, sonar_wall_t, sonar_ped_h + sonar_wall_h + 0.01]);
        }
        // front and rear lips on top of the towers only (channel stays open front-to-back below the case)
        for (sy = [-1, 1], xl = [x0, x1 - sonar_lip])
            translate([xl, sy > 0 ? pocket_w / 2 - sonar_foot : -pocket_w / 2, z0 + sonar_ped_h - 0.01])
                cube([sonar_lip, sonar_foot, 5 + 0.01]);
    }
}

module ribs() {
    for (y = [-rib_y, rib_y])
        translate([-plate_l / 2 + 10, y - rib_t / 2, plate_z - rib_h])
            cube([plate_l - 20, rib_t, rib_h + 0.01]);
}

module outline_groove(size, pos) {   // 0.6 mm deep, 1 mm wide groove around a footprint, as a placement guide
    translate([pos[0], pos[1], plate_z + plate_t - 0.6])
        linear_extrude(1)
            difference() {
                offset(1) square([size[0] + clr, size[1] + clr], center = true);
                square([size[0] + clr, size[1] + clr], center = true);
            }
}

module plate_cutouts() {
    outline_groove([bb_l, bb_w], bb_pos);
    outline_groove([bat_l, bat_w], bat_pos);
    // wheel wells: notch the plate edge around each wheel
    for (sy = [-1, 1])
        translate([motor_x - (wheel_r + well_margin), sy > 0 ? motor_y + 0.5 : -(plate_w / 2 + 1), plate_z - 1])
            cube([2 * (wheel_r + well_margin), plate_w / 2 + 1 - (motor_y + 0.5) + 0.01, plate_t + 2]);
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
            for (s = [-1, 1]) motor_mount(s);
            caster_mount();
            sonar_cradle();
            // PRINT ORIENTATION: top face on the bed, ears pointing up, no supports.
            // That only works while the top face is flat, so the locating lips below are
            // off by default. Velcro / adhesive holds the breadboard, zip ties the battery.
            if (lips) {
                side_rails([bb_l, bb_w], bb_pos);
                difference() {
                    lip([bat_l, bat_w], bat_pos);
                    // wire notch: cut the lip at the chosen corner, on the X-facing end
                    translate([bat_pos[0] + bat_wire_corner[0] * (bat_l / 2 + 2), bat_pos[1] + bat_wire_corner[1] * (bat_w / 2 - bat_wire_notch / 2 - 2), plate_z + plate_t - 1])
                        linear_extrude(lip_h + 2) square([2 * lip_t + clr + 4, bat_wire_notch], center = true);
                }
            }
        }
        plate_cutouts();
    }
}

show_ghosts = is_undef(coupon_only);   // ghosts only with the full chassis preview
if (is_undef(coupon_only)) chassis();
if (show_ghosts) for (s = [-1, 1]) motor_ghost(s);
// ghost: breadboard and the sonar case standing on its front rows
if (show_ghosts) %translate([bb_pos[0], bb_pos[1], plate_z + plate_t]) linear_extrude(bb_h) square([bb_l, bb_w], center = true);
if (show_ghosts) %translate([sonar_x_front - sonar_lip - sonar_case_d / 2 - clr / 2, sonar_y, plate_z + plate_t + sonar_ped_h]) linear_extrude(23.4) square([sonar_case_d, sonar_case_w], center = true);
if (show_ghosts) %translate([bat_pos[0], bat_pos[1], plate_z + plate_t]) linear_extrude(bat_h) square([bat_l, bat_w], center = true);
if (show_ghosts) %translate([caster_x, 0, 0]) { translate([0, 0, caster_h - 1]) linear_extrude(1) square(caster_plate, center = true); translate([6, 0, 12.5]) rotate([90, 0, 0]) cylinder(d = 25, h = 10, center = true); }
