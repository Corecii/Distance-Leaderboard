#!/bin/bash
export MSYS_NO_PATHCONV=1;

SCRIPT_DIR=$(dirname "$0")

CONTAINER_NAME=distance-evaluator

docker container kill $CONTAINER_NAME;
docker container rm $CONTAINER_NAME;

docker build -t $CONTAINER_NAME $SCRIPT_DIR

docker run \
	--mount type=bind,source="$(realpath $SCRIPT_DIR/../shared)",target="/home/user/output" \
	-p 6080:6080 \
	--name $CONTAINER_NAME \
	--env-file $SCRIPT_DIR/.env \
	$CONTAINER_NAME && \
sh $SCRIPT_DIR/../browser/run.sh