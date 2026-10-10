// The sonar cradle as its own part: towers, end walls and lips on a 2 mm base, with an M3 bolt
// hole down through each tower (head recessed into the tower top, under the case) matching the
// two holes in the chassis plate. Prints base-down.
include <chassis.scad>
coupon_only = true;
show_ghosts = false;
base_t = 2;
module holder() {
    pocket_d = sonar_case_d + clr;  pocket_w = sonar_case_w + clr;
    x1 = sonar_x_front; x0 = x1 - pocket_d - 2 * sonar_lip;
    // bring the chassis-frame cradle down onto the base
    translate([-x0, -sonar_y, -(plate_z + plate_t) + base_t])
        difference() {
            union() {
                sonar_cradle();
                translate([x0, sonar_y - pocket_w / 2 - sonar_wall_t, plate_z + plate_t - base_t])
                    cube([pocket_d + 2 * sonar_lip, pocket_w + 2 * sonar_wall_t, base_t + 0.01]);
            }
            sonar_cradle_bolt_cuts();
        }
}
holder();
