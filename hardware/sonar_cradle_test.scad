// Quick print to validate the sonar cradle: both towers, end walls and lips on a patch of plate.
// Drop the assembled case in with its jumper connectors on; the connectors hang in the channel.
include <chassis.scad>
coupon_only = true;
show_ghosts = false;
module coupon() {
    intersection() {
        chassis();
        translate([sonar_x_front - sonar_case_d - 2 * sonar_lip - clr - 8, sonar_y - sonar_case_w / 2 - sonar_wall_t - 6, 0])
            cube([sonar_case_d + 2 * sonar_lip + clr + 12, sonar_case_w + 2 * sonar_wall_t + 12, 100]);
    }
}
coupon();
