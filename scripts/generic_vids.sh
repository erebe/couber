#!/usr/bin/env bash

set -e
video_url=${1}
video_name=${3}
mkdir "${video_name}"

video_path=${video_name}/${video_name}

yt-dlp -o ${video_path}.mp4 ${video_url}
ffmpegthumbnailer -i "${video_path}.mp4" -o "${video_path}.thumbnail.png" -s 500

cat <<EOF > ${video_path}.js
{
  "name": "${video_name}",
  "url": "/videos/${video_path}.mp4",
  "tags": [],
  "original": "/videos/${video_path}.mp4",
  "thumbnail": "/videos/${video_path}.thumbnail.png",
  "creation_timestamp": $(date '+%s')
}
EOF

rm -rf "${2}/${video_name}"
mv "${video_name}" "${2}"
