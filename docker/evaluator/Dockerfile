FROM corecii/steam-client:ubuntu1804

RUN sudo apt-get install -fyqq libssl1.1 libssl-dev && \
	sudo apt-get install -fyqq libc6

RUN sudo apt-get install inotify-tools

COPY etc /etc

COPY home /home

RUN sudo chmod +x /home/user/entry.sh && \
	sudo chmod +x /home/user/distance_leaderboard_evaluator && \
	sudo chmod +x /home/user/libsteam_api.so && \
	sudo chown -v user:user /home/user && \
	sudo chmod -v 777 /home/user

CMD /home/user/entry.sh