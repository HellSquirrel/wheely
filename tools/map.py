#!/usr/bin/env python3
"""Draw a map from wheely's Wi-Fi telemetry by dead reckoning.

There are no wheel encoders yet, so the pose is integrated from the COMMANDED wheel speeds:
distance = speed/1000 * V_FULL * dt, heading change from the speed difference over the track
width. That drifts, and the two calibration constants below need measuring once:

  V_FULL_MM_S   how fast the robot moves at speed 1000 on your floor (drive a measured metre)
  TRACK_MM      distance between the wheel centres

Obstacle points are placed SONAR_OFFSET_MM ahead of the axle centre, along the heading, at
the measured distance. Needs matplotlib: pip install matplotlib

    python3 tools/map.py                 # live from udp/4210
    python3 tools/map.py --csv run1.csv  # replay a tools/telemetry.py log
"""
import argparse, csv, json, math, socket, sys, time

V_FULL_MM_S = 500.0     # calibrate: at speed 1000 on your floor
TRACK_MM = 170.0        # wheel centre to wheel centre
SONAR_OFFSET_MM = 150.0 # axle centre to sonar face
MAX_OBSTACLE_MM = 2000  # ignore readings further than this (walls far away are smeared)


class DeadReckoner:
    def __init__(self):
        self.x = self.y = 0.0
        self.theta = 0.0           # radians, 0 = +x, counter-clockwise positive
        self.last_t = None
        self.path = [(0.0, 0.0)]
        self.obstacles = []

    def feed(self, s):
        t = s["t"] / 1000.0
        if self.last_t is not None:
            dt = max(0.0, min(t - self.last_t, 1.0))  # cap gaps so a lost packet doesn't teleport us
            vl = s["l"] / 1000.0 * V_FULL_MM_S
            vr = s["r"] / 1000.0 * V_FULL_MM_S
            v = (vl + vr) / 2.0
            w = (vr - vl) / TRACK_MM
            self.theta += w * dt
            self.x += v * math.cos(self.theta) * dt
            self.y += v * math.sin(self.theta) * dt
            if v or w:
                self.path.append((self.x, self.y))
        self.last_t = t
        mm = s.get("mm")
        if mm is not None and 0 < mm < MAX_OBSTACLE_MM:
            d = SONAR_OFFSET_MM + mm
            self.obstacles.append((self.x + d * math.cos(self.theta), self.y + d * math.sin(self.theta)))


def samples_from_udp(port):
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(("", port))
    sock.settimeout(0.05)
    print(f"listening on udp/{port}", file=sys.stderr)
    while True:
        try:
            data, _ = sock.recvfrom(2048)
        except socket.timeout:
            yield None
            continue
        try:
            yield json.loads(data)
        except json.JSONDecodeError:
            pass


def samples_from_csv(path):
    with open(path, newline="") as f:
        for row in csv.DictReader(f):
            yield {
                "t": float(row["t"]),
                "l": float(row["l"]),
                "r": float(row["r"]),
                "mm": int(row["mm"]) if row["mm"] not in ("", "None") else None,
            }


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--port", type=int, default=4210)
    ap.add_argument("--csv", help="replay a telemetry.py log instead of listening")
    args = ap.parse_args()
    try:
        import matplotlib.pyplot as plt
    except ImportError:
        sys.exit("matplotlib is needed: pip install matplotlib")

    dr = DeadReckoner()
    plt.ion()
    fig, ax = plt.subplots(figsize=(7, 7))
    ax.set_aspect("equal")
    ax.set_xlabel("mm")
    ax.set_ylabel("mm")
    ax.set_title("wheely: path (green) and sonar hits (red)")
    path_line, = ax.plot([], [], "-", color="tab:green", lw=1.5)
    obst = ax.scatter([], [], s=6, color="tab:red")
    robot, = ax.plot([], [], "o", color="tab:blue", ms=8)
    ax.grid(True, alpha=0.3)

    source = samples_from_csv(args.csv) if args.csv else samples_from_udp(args.port)
    last_draw = 0.0
    for s in source:
        if s is not None:
            dr.feed(s)
        now = time.time()
        if now - last_draw < 0.2 and s is not None:
            continue
        last_draw = now
        xs, ys = zip(*dr.path)
        path_line.set_data(xs, ys)
        if dr.obstacles:
            obst.set_offsets(dr.obstacles)
        robot.set_data([dr.x], [dr.y])
        ax.relim()
        ax.autoscale_view()
        fig.canvas.draw_idle()
        fig.canvas.flush_events()
        if not plt.fignum_exists(fig.number):
            break
    if args.csv:
        plt.ioff()
        plt.show()


if __name__ == "__main__":
    main()
