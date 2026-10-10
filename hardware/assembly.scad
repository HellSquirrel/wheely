// Assembled view: the v15 plate with the separate cradle in place and every part where it goes.
// Preview only (colours); not a print file.
include <chassis.scad>
coupon_only = true;
show_ghosts = false;

color("SteelBlue") chassis();
color("SteelBlue") translate([0, 0, 2 - 2]) {   // cradle sits on the plate top (its 2 mm base is modelled on the plate top)
    sonar_cradle();
    pocket_d = sonar_case_d + clr; pocket_w = sonar_case_w + clr;
    translate([sonar_x_front - pocket_d - 2 * sonar_lip, sonar_y - pocket_w / 2 - sonar_wall_t, plate_z + plate_t])
        cube([pocket_d + 2 * sonar_lip, pocket_w + 2 * sonar_wall_t, 2]);
}

// motors: gearbox (yellow) + can (silver), shaft at motor_x, cans forward
for (side = [-1, 1]) {
    gb_y = side * (motor_y - tt_thk / 2);
    color("Gold") translate([motor_x - tt_end_to_shaft, gb_y - tt_thk / 2, wheel_r - tt_w / 2]) cube([tt_gear_len, tt_thk, tt_w]);
    color("Silver") translate([motor_x - tt_end_to_shaft + tt_gear_len, gb_y, wheel_r]) rotate([0, 90, 0]) cylinder(d = 19, h = tt_len - tt_gear_len);
    color("White") translate([motor_x, side * (motor_y + 2 + wheel_w / 2), wheel_r]) rotate([90, 0, 0]) cylinder(d = wheel_d, h = wheel_w, center = true);
    color("DimGray") translate([motor_x, side * (motor_y + 2 + wheel_w / 2), wheel_r]) rotate([90, 0, 0]) cylinder(d = wheel_d + 2, h = wheel_w - 4, center = true);
}
// caster: plate under the spacer block, swivel, 25 mm wheel touching the floor
color("Silver") translate([caster_x, 0, caster_h - 1.5]) linear_extrude(1.5) square(caster_plate, center = true);
color("Silver") translate([caster_x, 0, 20]) cylinder(d = 8, h = caster_h - 1.5 - 20);
color("WhiteSmoke") translate([caster_x + 6, 0, 12.5]) rotate([90, 0, 0]) cylinder(d = 25, h = 10, center = true);
// breadboard + Pico, battery box, sonar case with transducers, jumper connectors in the channel
color("WhiteSmoke") translate([bb_pos[0], bb_pos[1], plate_z + plate_t]) linear_extrude(bb_h) square([bb_l, bb_w], center = true);
color("DarkGreen") translate([bb_pos[0] - bb_l / 2 + 30, bb_pos[1], plate_z + plate_t + bb_h + 8]) linear_extrude(1.5) square([51, 21], center = true);
color("Black") translate([bat_pos[0], bat_pos[1], plate_z + plate_t]) linear_extrude(bat_h) square([bat_l, bat_w], center = true);
case_z = plate_z + plate_t + 2 + sonar_ped_h;
color("LightSalmon") translate([sonar_x_front - sonar_lip - sonar_case_d - clr / 2, sonar_y - sonar_case_w / 2, case_z]) cube([sonar_case_d, sonar_case_w, 23.4]);
for (sy = [-1, 1]) color("Silver") translate([sonar_x_front - sonar_lip, sonar_y + sy * 13, case_z + 12]) rotate([0, 90, 0]) cylinder(d = 16, h = 2);
color("Black") translate([sonar_x_front - sonar_lip - sonar_case_d / 2 - 1.5, sonar_y - 5, case_z - 10]) cube([3, 10, 10]);
