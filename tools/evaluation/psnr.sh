#!/bin/bash

export TMP=$(mktemp)
if [ "$target" = "rwc" ]; then
    ../../../tmp/VVCSoftware_VTM/bin/DecoderAppStatic -b $compressed -o ${compressed}.yuv 2>/dev/null > /dev/null
else
    ffmpeg -i $compressed -f rawvideo -pix_fmt yuv420p ${compressed}.yuv -y 2>/dev/null > /dev/null
fi
ffmpeg -f rawvideo -pix_fmt yuv420p -s ${width}x${height} -r ${frame_rate} -i ${compressed}.yuv -i $original -lavfi psnr=stats_file=$TMP -an -f null - 2>/dev/null >/dev/null
rm -f ${compressed}.yuv 2>/dev/null
cat $TMP | sed -e 's/.*psnr_avg:\([^ ]*\).*psnr_y:\([^ ]*\).*psnr_u:\([^ ]*\).*psnr_v:\([^ ]*\).*/\{"Avg":\1,"Y":\2,"U":\3,"V":\4\}/g' | jq -s .
