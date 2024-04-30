FROM node:16.17.1 AS node-build

WORKDIR /usr/src/airdo
COPY ./ ./
RUN npm install
RUN npm run build




FROM rust:latest AS rust-build

WORKDIR /usr/src/airdo
COPY ./ ./
WORKDIR /usr/src/airdo/src-api
RUN mv config/config-example.yml config/config.yml
RUN cargo build --release




FROM debian:bookworm-slim as proxy-build

WORKDIR /usr/src/airdo
RUN apt-get update
RUN apt-get install -y wget
RUN wget https://github.com/SagerNet/sing-box/releases/download/v1.8.12/sing-box-1.8.12-linux-amd64.tar.gz
RUN tar -zxvf sing-box-1.8.12-linux-amd64.tar.gz
RUN mv sing-box-1.8.12-linux-amd64 sing-box
RUN wget https://github.com/MetaCubeX/mihomo/releases/download/v1.18.3/mihomo-linux-amd64-v1.18.3.gz
RUN mkdir mihomo
RUN gzip -dN mihomo-linux-amd64-v1.18.3.gz
RUN mv mihomo-linux-amd64 mihomo/mihomo






FROM debian:bookworm-slim

WORKDIR /airdo

COPY --from=node-build /usr/src/airdo/dist /airdo/ui
COPY --from=rust-build /usr/src/airdo/src-api/target/release/airdo /airdo/airdo
COPY --from=rust-build /usr/src/airdo/src-api/config /airdo/config
COPY --from=proxy-build /usr/src/airdo/sing-box /airdo/sing-box
COPY --from=proxy-build /usr/src/airdo/mihomo /airdo/mihomo

RUN chmod +x /airdo/mihomo/mihomo /airdo/sing-box/sing-box
RUN apt-get update
RUN apt-get install -y openssl ca-certificates

CMD exec /airdo/airdo