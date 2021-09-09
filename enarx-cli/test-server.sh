#!/bin/bash

SOURCE_PATH=$(dirname ${BASH_SOURCE[0]})
SOCKET_PATH=$HOME/enarx.sock
BINARY_PATH=$SOURCE_PATH/../target/debug/enarx-cli

cargo build && { 
	systemd-socket-activate -l $SOCKET_PATH -a \
	$BINARY_PATH --log-filter=enarx_cli=debug serve --systemd-socket-accept $SOCKET_PATH &
} && sleep 0.1 && echo 'yay' | nc -U $SOCKET_PATH
jobs
