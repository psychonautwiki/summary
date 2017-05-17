FROM kenansulayman/rust-nightly:latest

# copy program files independently
ADD src /summary/src
ADD Cargo.toml /summary/Cargo.toml
ADD Cargo.lock /summary/Cargo.lock

RUN cd /summary && cargo build -v --release

# copy wordnet dictionary
ADD dict /summary/dict

ENV ROCKET_ENV prod
ENV ROCKET_PORT 80
ENV ROCKET_ADDRESS 0.0.0.0

CMD ["/summary/target/release/summary"]