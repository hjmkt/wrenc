#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cargo build --release
ffmpeg -i $SCRIPT_DIR/../assets/bus_352x288_30fps_30fr.mp4 -f rawvideo -pix_fmt yuv420p - | cargo run --release --bin wrenc -- -i - --input-size 352x288 --num-pictures 30 --qp 20 --output-size 352x288 --reconst $SCRIPT_DIR/../reconstructed.yuv -o $SCRIPT_DIR/../encoded.vvc
$VTM_ROOT/DecoderAppStatic -b $SCRIPT_DIR/../encoded.vvc -o $SCRIPT_DIR/../decoded.yuv

if cmp -s $SCRIPT_DIR/../reconstructed.yuv $SCRIPT_DIR/../decoded.yuv; then
    echo "[PASS] The reconstructed video frames are identical to the decoded video frames!"
    exit 0
else
    echo "[FAIL] The reconstructed video frames are NOT identical to the decoded video frames!"
    exit 1
fi
