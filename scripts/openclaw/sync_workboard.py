#!/usr/bin/env python3

from workboard_lib import sync_workboard


def main() -> int:
    stats = sync_workboard()
    print(
        "synced WORKBOARD.md: manual_open={manual_open} scout_open={scout_open}".format(
            manual_open=stats["manual_open"],
            scout_open=stats["scout_open"],
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
