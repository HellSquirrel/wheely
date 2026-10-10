// Upright bracket for the hinged HC-SR04 case (MakerWorld 1913595, modified for 22.5 mm transducer
// centres). The case's flat back has two ears with O4 holes 58 mm apart, 25.3 mm above the case's
// bottom edge. The bracket's face carries those holes; a window below lets the connectors and wires
// through to the breadboard. The foot bolts to the two holes already in the chassis plate.
// Standalone part. Print with the FACE FRONT on the bed: foot and gussets then stand up, no supports.
include <chassis.scad>
coupon_only = true;
show_ghosts = false;

case_hole_sep   = 58;     // ear holes, centre to centre
case_hole_d     = 3.4;    // M3 through the case's O4 holes (washers) ; use 4.3 for M4
case_bottom_below_holes = 25.3;
case_clear      = 20;     // case bottom edge this far above the plate: room for the connectors
face_t   = 3;   face_w = 70;  face_h = case_clear + case_bottom_below_holes + 5;   // holes 5 mm below the top edge
foot_t   = 3;   foot_x0 = 76; foot_x1 = plate_l / 2 - 2.5;                         // foot on the plate, 76..100
window_w = 24;  window_h = 15;                                                      // wire window in the face
gusset_t = 3;   gusset_y = 25; gusset_back = 18; gusset_up = 34;

hole_z = case_clear + case_bottom_below_holes;   // 45.3 above the plate top

module bracket() {
    difference() {
        union() {
            // foot on the plate
            translate([foot_x0, -face_w / 2, 0]) cube([foot_x1 - foot_x0, face_w, foot_t]);
            // face at the front of the foot
            translate([foot_x1 - face_t, -face_w / 2, 0]) cube([face_t, face_w, face_h]);
            // two gussets behind the face
            for (sy = [-1, 1])
                translate([foot_x1 - face_t, sy * gusset_y - gusset_t / 2, 0])
                    rotate([90, 0, 0]) mirror([0, 0, sy > 0 ? 0 : 1]) translate([0, 0, 0])
                        linear_extrude(gusset_t) polygon([[0, foot_t - 0.01], [0, gusset_up], [-gusset_back, foot_t - 0.01]]);
        }
        // case ear holes
        for (sy = [-1, 1])
            translate([foot_x1 - face_t - 1, sy * case_hole_sep / 2, hole_z]) rotate([0, 90, 0]) cylinder(d = case_hole_d, h = face_t + 2);
        // wire window
        translate([foot_x1 - face_t - 1, -window_w / 2, foot_t]) cube([face_t + 2, window_w, window_h]);
        // foot holes matching the plate
        for (sy = [-1, 1])
            translate([sonar_bolt_x, sy * sonar_bolt_y, -1]) cylinder(d = hole_m3, h = foot_t + 2);
    }
}
// positioned on the plate (chassis frame) when previewed from chassis-assembly; standalone here at the origin
translate([-foot_x0, 0, 0]) bracket();
