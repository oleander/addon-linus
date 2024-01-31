#!/bin/sh

source .env

echo "ping" | nc -w 1 localhost ${SOCKET_PORT} | grep -q "pong" || exit 1
