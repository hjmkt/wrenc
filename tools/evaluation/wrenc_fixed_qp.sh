#!/bin/bash

ffmpeg -i $input -f rawvideo -vframes $frames - 2>/dev/null | cargo run --release --bin wrenc -- -i - --input-size ${width}x${height} --num-pictures $frames --qp $qp --output-size ${width}x${height} ${max_split_depth} ${extra_params} -o $output 2>/dev/null
du -b $output | cut -f1
