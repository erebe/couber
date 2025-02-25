#!/usr/bin/env bash

set -e
video_url=${1}
video_name=${3}
mkdir "${video_name}"

video_path=${video_name}/${video_name}

yt-dlp -o ${video_path} ${video_url}
video_full=$(find ${video_name}/ -type f -name "${video_name}.*")
ffmpegthumbnailer -i "${video_full}" -o "${video_path}.thumbnail.png" -s 500

cat <<EOF > ${video_path}.js
{
  "name": "${video_name}",
  "url": "/videos/${video_full}",
  "tags": [],
  "original": "${video_url}",
  "thumbnail": "/videos/${video_path}.thumbnail.png",
  "creation_timestamp": $(date '+%s')
}
EOF

rm -rf "${2}/${video_name}"
mv "${video_name}" "${2}"
