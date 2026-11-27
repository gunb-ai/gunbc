#!/bin/bash
# john: Strata 8c5fafc1 experimental Pascal/Volta build (P100 sm_60, V100 sm_70), queued after the clock sweep so the compile does not skew GPU/thermal tests.
# Mirrors ken's CMakeCache STRATA_* options except STRATA_EXPERIMENTAL_SM60=ON and CMAKE_CUDA_ARCHITECTURES=60;70; CUDA 12.x because CUDA 13 dropped sm_60/sm_70.
# (queue wait removed: operator 01:43Z, build now)
set -e; T=~/strata-test; L=$T/build-sm60.log; exec > >(tee -a $L) 2>&1
echo "$(date -u +%FT%TZ) venv"; python3 -m venv $T/strata/.venv; $T/strata/.venv/bin/pip install -q -r $T/strata/requirements.txt
export PATH=/usr/local/cuda-12.9/bin:$T/strata/.venv/bin:$PATH
cd $T/strata-main
echo "$(date -u +%FT%TZ) configure"; cmake -B build-sm60 -G Ninja -DCMAKE_BUILD_TYPE=Release -DSTRATA_ENABLE_CUDA=ON -DSTRATA_EXPERIMENTAL_SM60=ON -DCMAKE_CUDA_ARCHITECTURES="60;70" -DSTRATA_MMQ_KQUANTS=ON -DSTRATA_MMQ_TQ_OVERLAY=ON -DSTRATA_NATIVE_EXPERTS=ON | tail -15
echo "$(date -u +%FT%TZ) build"; time cmake --build build-sm60 -j 64 | tail -5
ls -la build-sm60/strata build-sm60/*.so; echo "$(date -u +%FT%TZ) STRATA BUILD DONE"
