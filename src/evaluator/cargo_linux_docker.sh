docker.exe container kill distance_leaderboard_evaluator_cargo;
docker.exe container rm distance_leaderboard_evaluator_cargo;

docker.exe build -t distance_leaderboard_evaluator_cargo ./cargo_linux_docker

SCRIPT_DIR=$(realpath $(dirname "$0"))

echo "$(wslpath -m $SCRIPT_DIR)"

docker.exe run \
	--rm \
	--user "$(id -u)":"$(id -g)" \
	-v "$(wslpath -m $SCRIPT_DIR)":/usr/src/myapp \
	-w /usr/src/myapp \
	--name "distance_leaderboard_evaluator_cargo" \
	distance_leaderboard_evaluator_cargo sh -c "export RUST_BACKTRACE=full; cargo $@"