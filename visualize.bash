#!/usr/bin/env bash

# quit python visualizations on exit
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
espmonitor /dev/ttyUSB0 |& grep 'INFO - [01]\+' --line-buffered | sed -e 's/\x1b\[[0-9;]*m//g' --unbuffered | sed 's/.*INFO - //' --unbuffered | sed -e 's/\r//' --unbuffered | while read line; do echo "$line" | python visualize.py & done
