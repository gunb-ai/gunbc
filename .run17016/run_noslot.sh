#!/usr/bin/env bash
# No-slot stage runner (stage 0 / fetch only): plain ssh, prints HOST/BOOT from the script itself.
REL="$1"; shift
ssh -n -o BatchMode=yes -o StrictHostKeyChecking=no -o HostName=srv1.tailecbe08.ts.net briansrls@srv1.tailecbe08.ts.net "cd \$HOME/scratch/bold-crane-719/run && bash $REL $*"
