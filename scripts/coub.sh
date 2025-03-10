#!/usr/bin/env bash

set -e
video_name=${1}
mkdir "${video_name}"
video_path="${video_name}/${video_name}"
video_url="https://coub.com/view/${video_name}"
output="${2}"

loops=50

function cleanup() {
  rm -f ${video_path}.{mp4.avi,txt} 
  if [ "$1" != "0" ]; then
    echo "Error $1 occurred on $2"
    rm -rf "${video_name}"
  fi
}

trap "cleanup $? $LINENO" EXIT INT TERM


yt-dlp -o ${video_path}.mp4 ${video_url}
yt-dlp -f html5-audio-high -o ${video_path}.mp3 ${video_url}

printf '\x00\x00' | dd of=${video_path}.mp4 bs=1 count=2 conv=notrunc
for i in `seq 1 "$loops"`; do echo "file '${1}.mp4'" >> ${video_path}.txt; done
ffmpeg -y  -hide_banner -t 30 -f concat -i ${video_path}.txt -i ${video_path}.mp3 -c copy -shortest -movflags faststart -c:a aac -b:a 128k "${video_path}".ori.mp4

# Version without watermark
echo '' > "${video_path}.txt"
python3 remove_watermark.py "${video_path}.mp4"
for i in `seq 1 "$loops"`; do echo "file '${1}.mp4.avi'" >> ${video_path}.txt; done
ffmpeg -y  -hide_banner -t 30 -f concat -i ${video_path}.txt -i ${video_path}.mp3 -c copy -shortest -movflags faststart -vcodec libx264 -c:a aac -b:a 128k "$video_path".mp4
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
rm -rf "${2}/${video_name}"
mv "${video_name}" "${2}"
