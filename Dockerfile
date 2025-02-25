# Rust backend
FROM rust:1.78 AS builder_backend

LABEL org.opencontainers.image.source=https://github.com/erebe/couber

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
FROM debian:bullseye-slim AS builder_frontend

RUN apt-get update && \
	curl -sL https://deb.nodesource.com/setup_10.x | bash - && \
	apt install -y npm nodejs build-essential

COPY frontend frontend
WORKDIR /frontend

RUN npm install && \
    npm run build

# Runner
FROM debian:bookworm-slim

RUN useradd -ms /bin/bash app && \
	apt-get update && \
	apt install -y --no-install-recommends ca-certificates sqlite3 python3 python3-numpy python3-opencv ffmpeg ffmpegthumbnailer curl jq && \
	curl -sL -o /usr/local/bin/yt-dlp https://github.com/yt-dlp/yt-dlp/releases/download/2025.02.19/yt-dlp_linux && chmod +x /usr/local/bin/yt-dlp && \
	apt-get clean && \
	rm -rf /var/lib/apt/lists 
WORKDIR /home/app

USER app

COPY --chown=app:app --from=builder_backend backend/target/release/backend couber
COPY --chown=app:app --from=builder_frontend frontend/dist dist
COPY --chown=app:app scripts scripts
RUN mkdir -p data/videos
COPY --chown=app:app db.sqlite data/db.sqlite

ENV DATABASE_PATH=/home/app/data/db.sqlite
ENV VIDEOS_PATH=/home/app/data/videos
ENV WEBAPP_PATH=/home/app/dist
ENV SCRIPTS_PATH=/home/app/scripts
ENV PORT=8080
ENV RUST_LOG=info
EXPOSE 8080

CMD  ./couber
