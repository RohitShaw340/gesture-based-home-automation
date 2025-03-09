run: build
  @echo 'Running server'
  cd target && ./server

build: rotator server
  @echo 'Built to ./target'

target:
  mkdir -p target

rotator: target
  @echo 'Building rotator'
  cd rotator && cargo build --release
  mv rotator/target/release/rotator target

server: target
  @echo 'Building server'
  cd server && go build .
  mv server/server target && cp server/servo_config.json target
