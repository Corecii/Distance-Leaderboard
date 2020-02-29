docker.exe container kill distance_leaderboard_evaluator_cargo;
docker.exe container rm distance_leaderboard_evaluator_cargo;

docker.exe build -t distance_leaderboard_evaluator_cargo ./cargo_linux_docker

echo "$(wslpath -w $PWD)"

docker.exe run \
	--rm \
	--user "$(id -u)":"$(id -g)" \
	-v "$(wslpath -w $PWD)":/usr/src/myapp \
	-w /usr/src/myapp \
	--name "distance_leaderboard_evaluator_cargo" \
	distance_leaderboard_evaluator_cargo sh -c "export RUST_BACKTRACE=full; cargo $@"