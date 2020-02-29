
SCRIPT_DIR=$(dirname "$0")

sh -c "nohup $SCRIPT_DIR/browser/run.sh > /dev/null 2>&1 &"
sh -c "nohup $SCRIPT_DIR/evaluator/run.sh > /dev/null 2>&1 &"