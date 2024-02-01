#!/bin/sh

source .env

echo "ping" | nc -w 1 localhost 10200 | grep -q "pong" || exit 1
