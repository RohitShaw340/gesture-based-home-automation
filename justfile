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
  cp -f rotator/target/release/rotator target/rotate_camera

server: target
  @echo 'Building server'
  cd Server && /usr/local/go/bin/go build .
  mv Server/server target && cp Server/servo_config.json target

clean:
  rm -rf target
