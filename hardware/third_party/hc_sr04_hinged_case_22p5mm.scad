// Hinged HC-SR04 case (MakerWorld 1913595) with the transducer openings moved 1.75 mm inboard
// each, for a sensor whose transducers are 3.5 mm closer together (22.5 mm centres instead of 26).
hole_r = 8.25; collar_r = 9.25;
old_x = 13.0; new_x = 13.0 - 1.75; hy = 14.0;
floor_bot = -4.55; floor_top = -2.75; collar_top = -1.25;
slot = [0.06, 6.89, 10.3, 4.0];
$fn = 96;
difference() {
    union() {
        difference() {
            import("case_mesh.stl");
            // remove the old collars (everything above the floor around the old centres)
            for (sx = [-1, 1]) translate([sx * old_x, hy, floor_top]) cylinder(r = collar_r + 0.15, h = 10);
        }
        // plug the old openings through the floor
        for (sx = [-1, 1]) translate([sx * old_x, hy, floor_bot]) cylinder(r = hole_r + 0.3, h = floor_top - floor_bot + 0.01);
        // new collars
        for (sx = [-1, 1]) translate([sx * new_x, hy, floor_top - 0.01])
            difference() {
                cylinder(r = collar_r, h = collar_top - floor_top + 0.01);
                translate([0, 0, -1]) cylinder(r = hole_r, h = 5);
            }
    }
    // new openings through floor and collars
    for (sx = [-1, 1]) translate([sx * new_x, hy, floor_bot - 1]) cylinder(r = hole_r, h = 10);
    // keep the small centre slot open
    translate([slot[0], slot[1], floor_bot - 1]) linear_extrude(10) square([slot[2], slot[3]], center = true);
}
