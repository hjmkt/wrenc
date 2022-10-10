#!/bin/bash

export TMP=$(mktemp)
if [ "$target" = "rwc" ]; then
    ../../VVCSoftware_VTM/bin/DecoderAppStatic -b $compressed -o ${compressed}.yuv 2>/dev/null > /dev/null
else
    ffmpeg -i $compressed -f rawvideo -pix_fmt yuv420p ${compressed}.yuv -y 2>/dev/null > /dev/null
fi
ffmpeg -f rawvideo -pix_fmt yuv420p -s ${width}x${height} -r ${frame_rate} -i ${compressed}.yuv -i $original -lavfi ssim=stats_file=$TMP -an -f null - 2>/dev/null >/dev/null
rm -f ${compressed}.yuv 2>/dev/null
cat $TMP | sed -e 's/.*Y:\([^ ]*\).*U:\([^ ]*\).*V:\([^ ]*\).*All:\([^ ]*\).*/\{"Avg":\4,"Y":\1,"U":\2,"V":\3\}/g' | jq -s .
