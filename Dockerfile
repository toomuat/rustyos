FROM rustlang/rust:nightly
WORKDIR /app
COPY . .
RUN cd loader && cargo build --release && \
    cd ../kernel && cargo build --release
