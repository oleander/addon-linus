set dotenv-load := true

TASK0 := "toggle the lights"
TASK1 := "toggle the lights"
TIMEOUT := "10"
SOCKET := "nc 127.0.0.1 8080"
DOCKER_IMAGE := "amd64-addon-linus"

instructions:
  cargo run --bin instructions
openai: instructions
  cargo run --bin openai
socket: instructions
  cargo run --bin socket
default: instructions
  cargo run --bin default
ask: instructions
  echo {{TASK0}} | {{SOCKET}}
  sleep {{TIMEOUT}}
  echo {{TASK1}} | {{SOCKET}}
server: instructions
  cargo run --bin websocket
docker-build:
  docker build -t {{DOCKER_IMAGE}} .
docker-run: docker-build
  docker run -ti --env-file .env -p 10200:10200 --rm {{DOCKER_IMAGE}}

