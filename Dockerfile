# Rust backend
FROM rust:1.47 AS builder_backend

LABEL org.opencontainers.image.source https://github.com/erebe/couber

COPY docker/dummy.rs backend/dummy.rs
COPY backend/Cargo.toml backend/Cargo.toml

WORKDIR /backend
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml && \
    cargo fetch && \
    cargo build --release

COPY backend .

RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml && \ 
   cargo build --release 


# VueJS Frontend
FROM rust:1.47 AS builder_frontend

RUN apt-get update && \
	curl -sL https://deb.nodesource.com/setup_10.x | bash - && \
	apt install -y nodejs build-essential 

COPY frontend frontend
WORKDIR /frontend

RUN npm install && \
    npm run build

# Runner
FROM debian:bullseye-slim

RUN useradd -ms /bin/bash app && \
	apt-get update && \
	apt install -y --no-install-recommends ca-certificates sqlite3 python3 python3-numpy python3-opencv ffmpeg ffmpegthumbnailer youtube-dl curl jq && \
	apt-get clean && \
	rm -rf /var/lib/apt/lists 
WORKDIR /home/app

COPY --from=builder_backend backend/target/release/backend couber
COPY --from=builder_frontend frontend/dist dist
COPY scripts scripts

ENV DATABASE_PATH=/home/app/db/db.sqlite
ENV VIDEOS_PATH=/home/app/videos
ENV WEBAPP_PATH=/home/app/dist
ENV SCRIPTS_PATH=/home/app/scripts
ENV PORT=8080
ENV RUST_LOG=info
EXPOSE 8080

VOLUME /home/app/videos
VOLUME /home/app/db

CMD chown -R app:app . && \
    runuser -u app ./couber
