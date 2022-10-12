#!/bin/bash

ffmpeg -i $input -vframes $frames -c:v libx264 -preset $preset -intra -g 1 -qp $qp $output -y 2>/dev/null
du -b $output | cut -f1
