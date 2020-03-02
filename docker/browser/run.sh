#!/bin/bash
export MSYS_NO_PATHCONV=1;

SCRIPT_DIR=$(dirname "$0")

CONTAINER_NAME=distance-evaluator-browser

docker container kill $CONTAINER_NAME;
docker container rm $CONTAINER_NAME;

docker build -t $CONTAINER_NAME $SCRIPT_DIR

# sudo ufw delete allow $PORT_HTTP
sudo ufw allow 80

docker run \
	--mount type=bind,source="$(realpath $SCRIPT_DIR/../shared)",target="/root/output" \
	-p 80:80 \
	--name $CONTAINER_NAME \
	--restart=unless-stopped \
	$CONTAINER_NAME