#!/bin/bash

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

jobs=$1

for i in `seq 1 $jobs`; do
    python optimize_bd_psnr.py &
    sleep 1
done

wait
