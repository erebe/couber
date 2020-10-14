#!/bin/sh


for i in $(curl http://api.erebe.eu/videos/ | grep '.mp4' | cut -d '"' -f2)
do
	name=$(echo $i | sed 's/__.mp4//')
	tags=$(curl -s 'https://coub.com/view/'$name | grep -A1 coubPageCoubJson | tail -n 1 | jq .tags[].value | paste -sd ',' | sed "s/'/''/g")
	echo "INSERT INTO videos (name, url, tags) VALUES ('$name', '$i','[$tags]');"

done
