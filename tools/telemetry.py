#!/usr/bin/env python3
"""Log wheely's Wi-Fi telemetry.

The robot broadcasts one JSON object per 100 ms on UDP port 4210 (see src/bin/remote.rs).
This prints a compact line per sample and appends every sample to a CSV file.

    python3 tools/telemetry.py            # listen on 4210, write telemetry-<timestamp>.csv
    python3 tools/telemetry.py --port 4210 --csv run1.csv

Stop with Ctrl-C. Standard library only.
"""
import argparse, csv, json, socket, sys, time

FIELDS = ["recv", "seq", "t", "mm", "sonar", "blocked", "cmd", "eff", "cruise", "speed", "l", "r", "ble"]


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--port", type=int, default=4210)
    ap.add_argument("--csv", default=time.strftime("telemetry-%Y%m%d-%H%M%S.csv"))
    args = ap.parse_args()

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(("", args.port))
    print(f"listening on udp/{args.port}, writing {args.csv}  (Ctrl-C to stop)", file=sys.stderr)

    with open(args.csv, "a", newline="") as f:
        w = csv.DictWriter(f, fieldnames=FIELDS, extrasaction="ignore")
        if f.tell() == 0:
            w.writeheader()
        last_seq, lost = None, 0
        try:
            while True:
                data, (host, _) = sock.recvfrom(2048)
                try:
                    s = json.loads(data)
                except json.JSONDecodeError:
                    print(f"bad packet from {host}: {data!r}", file=sys.stderr)
                    continue
                s["recv"] = round(time.time(), 3)
                w.writerow(s)
                f.flush()
                if last_seq is not None and s["seq"] != last_seq + 1:
                    lost += 1
                last_seq = s["seq"]
                mm = f'{s["mm"]:4d}mm' if s.get("mm") is not None else f'{s.get("sonar", "?"):>6}'
                print(f'{s["t"]/1000:8.1f}s  {mm}  {"BLOCKED" if s["blocked"] else "clear  "}  '
                      f'{s["cmd"]:>7} -> {s["eff"]:<7} cruise={s["cruise"]:<8} L{s["l"]:5d} R{s["r"]:5d}  '
                      f'{"ble" if s["ble"] else "   "}  gaps={lost}', end="\r" if sys.stdout.isatty() else "\n")
        except KeyboardInterrupt:
            print(file=sys.stderr)


if __name__ == "__main__":
    main()
