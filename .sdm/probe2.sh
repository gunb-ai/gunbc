#!/usr/bin/env bash
echo "PROBE-ONE $(date -u +%H:%M:%S)"
echo "cwd=$PWD"; ls -la .sdm | head -8
echo "PROBE-TWO $(date -u +%H:%M:%S)"; sleep 20
echo "PROBE-THREE $(date -u +%H:%M:%S)"; sleep 20
echo "PROBE-DONE $(date -u +%H:%M:%S)"
