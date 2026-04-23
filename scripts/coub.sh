#!/usr/bin/env bash

set -e
video_name=${1}
video_path="${video_name}/${video_name}"
video_url="https://coub.com/view/${video_name}"

loops=50

rm -rf "${video_name}"
mkdir "${video_name}"


yt-dlp -o ${video_path}.mp4 ${video_url}
yt-dlp -f html5-audio-high -o ${video_path}.mp3 ${video_url}

printf '\x00\x00' | dd of=${video_path}.mp4 bs=1 count=2 conv=notrunc
for i in `seq 1 "$loops"`; do echo "file '${1}.mp4'" >> ${video_path}.txt; done
ffmpeg -y  -hide_banner -t 30 -f concat -i ${video_path}.txt -i ${video_path}.mp3 -c copy -shortest -movflags faststart -c:a aac -b:a 192k "${video_path}".ori.mp4

# Version without watermark
echo '' > "${video_path}.txt"
python3 remove_watermark.py "${video_path}.mp4"
for i in `seq 1 "$loops"`; do echo "file '${1}.mp4.avi'" >> ${video_path}.txt; done
ffmpeg -y  -hide_banner -t 30 -f concat -i ${video_path}.txt -i ${video_path}.mp3 -c copy -shortest -movflags faststart -c:v libx265 -crf 24 -preset slow -tag:v hvc1 -c:a aac -b:a 192k "$video_path".mp4
ffmpegthumbnailer -i "${video_path}.mp4" -o "${video_path}.thumbnail.png" -s 500

tags=$(curl -s ${video_url} | grep -A1 coubPageCoubJson | tail -n 1 | jq .tags[].value | paste -sd ',')

cat <<EOF > ${video_path}.js
{
  "name": "${video_name}",
  "url": "/videos/${video_path}.mp4",
  "tags": [${tags}],
  "original": "/videos/${video_path}.ori.mp4",
  "thumbnail": "/videos/${video_path}.thumbnail.png",
  "creation_timestamp": $(date '+%s')
}
EOF

cleanup 0 ''
