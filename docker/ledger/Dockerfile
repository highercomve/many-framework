FROM many_local/repo_build:latest as builder

# ==== STAGE 2 ====
FROM debian:bullseye as runtime

RUN useradd -ms /bin/bash many

# Contains the ledger.db store.
VOLUME /persistent
VOLUME /state

STOPSIGNAL SIGTERM

USER many
WORKDIR /app

EXPOSE 8000

COPY --from=builder /src/target/release/many-ledger /app/many-ledger

ENTRYPOINT ["./many-ledger"]
CMD ["--addr", "0.0.0.0:8000", "-v", "-v", "--state", "/state/state.json", "--persistent", "/persistent/ledger.db", "--pem", "/state/id.pem"]
