#!/usr/bin/env python3
"""Compatibility entry point for the current Phase 3C campaign validator."""

from __future__ import annotations

import sys

from validate_campaign_v2 import main


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
