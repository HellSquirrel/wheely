// Quick print to validate the caster mount: spacer block with its four slots and the
// counterbores in the plate above, on a patch of plate. Bolt the caster on, check it sits square.
include <chassis.scad>
coupon_only = true;
show_ghosts = false;
module coupon() {
    difference() {
        intersection() {
            chassis();
            translate([caster_x - (caster_plate + 4) / 2 - 6, -(caster_plate + 4) / 2 - 6, 0])
                cube([caster_plate + 4 + 12, caster_plate + 4 + 12, 100]);
        }
        // the sonar cradle starts above the plate just ahead of the block: shave everything above
        // the plate surface in that zone so the coupon is only the caster mount
        translate([caster_x + 10, -100, plate_z + plate_t]) cube([100, 200, 100]);
    }
}
coupon();
