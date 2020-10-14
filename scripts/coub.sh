#!/usr/bin/env bash

set -e
mkdir "${1}"
video_path="$1/$1"
video_url="https://coub.com/view/$1"

loops=50

function cleanup() {
  rm -f ${video_path}.{mp4.avi,txt,mp3} 
}

trap cleanup EXIT INT TERM


youtube-dl -o ${video_path}.mp4 ${video_url}
youtube-dl -f html5-audio-high -o ${video_path}.mp3 ${video_url}

printf '\x00\x00' | dd of=${video_path}.mp4 bs=1 count=2 conv=notrunc
for i in `seq 1 "$loops"`; do echo "file '${1}.mp4'" >> ${video_path}.txt; done
ffmpeg -y  -hide_banner -t 30 -f concat -i ${video_path}.txt -i ${video_path}.mp3 -c copy -shortest -movflags faststart -c:a aac -b:a 128k "${video_path}".ori.mp4

# Version without watermark
echo '' > "${video_path}.txt"
python3 remove_watermark.py "${video_path}.mp4"
for i in `seq 1 "$loops"`; do echo "file '${1}.mp4.avi'" >> ${video_path}.txt; done
ffmpeg -y  -hide_banner -t 30 -f concat -i ${video_path}.txt -i ${video_path}.mp3 -c copy -shortest -movflags faststart -vcodec libx264 -c:a aac -b:a 128k "$video_path".mp4
 
tags=$(curl -s ${video_url} | grep -A1 coubPageCoubJson | tail -n 1 | jq .tags[].value | paste -sd ',')

cat <<EOF > ${1}.js
{
  "name": "${1}",
  "url": "${video_url}",
  "tags": [${tags}]
}
EOF
