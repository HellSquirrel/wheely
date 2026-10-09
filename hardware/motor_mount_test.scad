// Quick print to validate the motor mount before printing the whole chassis.
// One mount (left side) on a patch of plate, with the wheel well edge as in the real part.
include <chassis.scad>
coupon_only = true;     // suppress the full chassis at top level
show_ghosts = false;    // and the preview ghosts
module coupon() {
    intersection() {
        chassis();
        // a box around the left motor mount: a bit of plate each way, and the well edge
        translate([motor_x + tt_hole_x - 30, motor_y - 32, 0]) cube([60, 40, 100]);
    }
}
coupon();
