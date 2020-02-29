export SCRIPT_DIR=$(dirname "$0")

rsync -azP --delete --exclude=.vscode --exclude=node_modules $SCRIPT_DIR/../src/browser/* $SCRIPT_DIR/browser/browser
rm $SCRIPT_DIR/evaluator/home/user/distance_leaderboard_evaluator;
cp $SCRIPT_DIR/../src/evaluator/target/debug/distance_leaderboard_evaluator $SCRIPT_DIR/evaluator/home/user/distance_leaderboard_evaluator
rm $SCRIPT_DIR/evaluator/home/user/libsteam_api.so;
cp $SCRIPT_DIR/../src/evaluator/target/debug/build/*/*/libsteam_api.so $SCRIPT_DIR/evaluator/home/user/libsteam_api.so