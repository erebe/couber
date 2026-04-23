#!/usr/bin/env bash

set -xe
video_url=${1}
video_name=${2}

function cleanup() {
  rm -rf "${video_name}"
}

rm -rf "${video_name}"
mkdir "${video_name}"
trap "cleanup $? $LINENO" EXIT INT TERM

video_path=${video_name}/${video_name}

yt-dlp -o ${video_path}.ori.mp4 ${video_url}
ffmpeg -i ${video_path}.ori.mp4 -c:v libx265 -crf 24 -preset slow -tag:v hvc1 -c:a aac -b:a 192k ${video_path}.mp4
ffmpegthumbnailer -i "${video_path}.mp4" -o "${video_path}.thumbnail.png" -s 500

cat <<EOF > ${video_path}.js
{
  "name": "${video_name}",
  "url": "/videos/${video_path}.mp4",
  "tags": [],
  "original": "${video_url}",
  "thumbnail": "/videos/${video_path}.thumbnail.png",
  "creation_timestamp": $(date '+%s')
}
EOF