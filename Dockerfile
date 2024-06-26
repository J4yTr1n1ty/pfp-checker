FROM rust:1.78-buster as build

# create a new empty shell project
RUN USER=root cargo new --bin pfp-checker
WORKDIR /pfp-checker

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# copy SQL info
COPY ./migrations ./migrations
COPY ./.sqlx ./.sqlx
COPY ./database.sqlite ./database.sqlite

# Install Database CLI and apply migrations
RUN cargo install sqlx-cli
RUN sqlx database setup --database-url sqlite:database.sqlite

# build for release
RUN rm ./target/release/deps/pfp_checker*
RUN cargo build --release

FROM rust:1.78-slim-buster

# copy the build artifact from the build stage
COPY --from=build /pfp-checker/target/release/pfp-checker .

# set the startup command to run your binary
CMD ["./pfp-checker"]
