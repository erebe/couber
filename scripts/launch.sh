#!/bin/bash


coubs=$(ls --color=none /mnt/nvme/default-couber-pv-claim-pvc-3f35ad4b-e68d-4090-ad83-7d625d49125a/videos)

echo $coubs  | parallel -d ' ' --bar --color -- ./coub.sh {} 

