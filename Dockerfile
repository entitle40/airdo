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
RUN apt-get install -y wget curl jq
RUN export SING_BOX_TAG_NAME=$(curl -s "https://api.github.com/repos/SagerNet/sing-box/releases/latest" | jq -r '.tag_name') && echo "SING_BOX_TAG_NAME: ${SING_BOX_TAG_NAME}" \
    && export SING_BOX_VERSION=${SING_BOX_TAG_NAME#?} && echo "SING_BOX_VERSION: ${SING_BOX_VERSION}" \
    && export MIHOMO_TAG_NAME=$(curl -s "https://api.github.com/repos/MetaCubeX/mihomo/releases/latest" | jq -r '.tag_name') && echo "MIHOMO_TAG_NAME: ${MIHOMO_TAG_NAME}" \
    && wget https://github.com/SagerNet/sing-box/releases/download/${SING_BOX_TAG_NAME}/sing-box-${SING_BOX_VERSION}-linux-amd64.tar.gz \
    && tar -zxvf sing-box-${SING_BOX_VERSION}-linux-amd64.tar.gz \
    && mv sing-box-${SING_BOX_VERSION}-linux-amd64 sing-box \
    && wget https://github.com/MetaCubeX/mihomo/releases/download/${MIHOMO_TAG_NAME}/mihomo-linux-amd64-${MIHOMO_TAG_NAME}.gz \
    && mkdir mihomo \
    && gzip -dN mihomo-linux-amd64-${MIHOMO_TAG_NAME}.gz \
    && mv mihomo-linux-amd64 mihomo/mihomo






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