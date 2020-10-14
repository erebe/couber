FROM rust:1.47 AS builder

COPY backend backend
COPY frontend frontend

# Rust backend
WORKDIR /backend
RUN cargo build --release

# Frontend
WORKDIR /frontend
RUN apt-get update && \
	curl -sL https://deb.nodesource.com/setup_10.x | bash - && \
	apt install -y nodejs  build-essential && \
	npm install && \
	npm run build


FROM debian:buster-slim

RUN useradd -ms /bin/bash app && \
	apt-get update && \
	apt install -y sqlite3 python3 python3-numpy python3-opencv ffmpeg youtube-dl curl jq && \
	apt-get clean && \
	rm -rf /var/lib/apt/lists
WORKDIR /home/app

COPY --from=builder backend/target/release/backend webserver
COPY --from=builder frontend/dist dist
COPY scripts scripts

ENV DATABASE_PATH=db/db.sqlite
ENV VIDEOS_PATH=./videos
ENV WEBAPP_PATH=./dist
ENV PORT=8080
ENV RUST_LOG=info
EXPOSE 8080

VOLUME /home/app/videos
VOLUME /home/app/db

CMD chown -R app:app . && \
    runuser -u app ./webserver
