// The sonar cradle as a standalone part: the same towers, end walls and lips as on the chassis,
// on a 2 mm base exactly its own footprint. No plate, no holes. Prints base-down.
include <chassis.scad>
coupon_only = true;
show_ghosts = false;
base_t = 2;
module holder() {
    pocket_d = sonar_case_d + clr;  pocket_w = sonar_case_w + clr;
    x1 = sonar_x_front; x0 = x1 - pocket_d - 2 * sonar_lip;
    // drop the chassis cradle onto a base: shift it so the plate top (where the cradle starts) becomes z = base_t
    translate([-x0, -sonar_y, -(plate_z + plate_t) + base_t]) sonar_cradle();
    translate([0, -pocket_w / 2 - sonar_wall_t, 0])
        cube([pocket_d + 2 * sonar_lip, pocket_w + 2 * sonar_wall_t, base_t + 0.01]);
}
holder();
