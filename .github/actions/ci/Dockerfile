FROM rust:1.37.0

LABEL "com.github.actions.name"="j2gbc Test"
LABEL "com.github.actions.description"="Container for testing "
LABEL "com.github.actions.icon"="play-circle"
LABEL "com.github.actions.color"="gray-dark"

ENV RUST_BACKTRACE=1

RUN apt-get update && apt-get install -y libasound-dev libgtk-3-dev

CMD cargo test
