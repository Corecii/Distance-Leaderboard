SCRIPT_DIR=$(realpath $(dirname "$0"))
sudo echo "0 0 * * fri root sh $SCRIPT_DIR/run.sh" > /etc/cron.d/distance-evaluator