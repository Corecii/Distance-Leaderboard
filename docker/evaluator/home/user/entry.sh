#!/bin/bash

sudo -E mkdir -p /home/user/.vnc
sudo -E sh -c 'echo $PW_VNC > /home/user/.vnc/passwdfile'

echo "WORKING: $TEST"

/usr/bin/supervisord -c /etc/supervisor/supervisord.conf &

export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/home/user/

echo "Waiting for steam to start..."

until pids=$(pidof steam)
do   
    sleep 1
done

echo "Waiting for steam to finish loading..."

while [ ! -f /home/user/.local/share/Steam/userdata/*/config/localconfig.vdf ]; do sleep 1; done

echo "Ready!"

sleep 5

rm /home/user/output/distance_leaderboard.db.temp
rm /home/user/output/distance_leaderboard.db.temp-journal

sudo -Hn -u user -i /home/user/distance_leaderboard_evaluator --file-db /home/user/output/distance_leaderboard.db.temp --file-officials /home/user/output/official_levels.json

rm /home/user/output/distance_leaderboard.db
mv /home/user/output/distance_leaderboard.db.temp /home/user/output/distance_leaderboard.db