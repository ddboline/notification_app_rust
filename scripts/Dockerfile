FROM rust:alpine3.14 as buildstage

RUN apk add musl-dev && mkdir -p /build/notification_app_rust

ADD Cargo.toml /build/notification_app_rust/
COPY src /build/notification_app_rust/src
COPY notification_app_api /build/notification_app_rust/notification_app_api
COPY notification_app_bot /build/notification_app_rust/notification_app_bot
COPY notification_app_lib /build/notification_app_rust/notification_app_lib

RUN mkdir -p /notification_app_rust && \
    cd /build/notification_app_rust && \
    cargo install --path=. --root=/notification_app_rust

FROM alpine:3.14

COPY --from=buildstage /notification_app_rust/bin/* /usr/bin/

STOPSIGNAL SIGTERM

CMD ["/usr/bin/notification-app-api"]
