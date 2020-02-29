FROM rust:1.41.0-stretch

RUN apt-get update
RUN apt-get install -yqq locate && updatedb
RUN apt-get install -yqq build-essential llvm-7-dev libclang-7-dev clang-7
RUN ln -s /usr/bin/clang-7 /usr/bin/clang && \
	ls /usr/bin

#RUN cp -r /usr/lib/llvm-3.8/lib/clang/3.8.1/include/* /usr/include