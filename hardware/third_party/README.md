# Third-party parts (modified)

- `hc_sr04_hinged_case_22p5mm.stl` / `.scad`: the MakerWorld model 1913595 "Hinged case for ultrasonic sensor
  HC-SR04", with the two transducer openings moved 1.75 mm inboard each (22.5 mm centres instead of 26) to suit
  the RCWL-1601 on this robot. The .scad imports `case_mesh.stl`, an STL export of the original 3MF, which is
  NOT committed here: download the original from MakerWorld and export it to regenerate. Licence of the original
  is whatever MakerWorld states for that model; this copy is for personal use on this project.
