FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef

WORKDIR /app

RUN apt update && apt install lld clang -y
FROM chef as planner
COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder

COPY --from=planner /app/recipe.json recipe.json

COPY . .

ENV SQLX_OFFLINE true

RUN cargo build --release --bin zero2prod
#runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app
RUN apt-get update -y \
		&& apt-get install -y --no-install-recommends openssl ca-certificates \
		# Clean up
		&& apt-get autoremove -y \
		&& apt-get clean -y \
		&& rm -rf /var/lib/apt/lists/*

#copy the compiled binary from the builder environment
#to our runtime environment
COPY --from=builder /app/target/release/zero2prod zero2prod

COPY configuration configuration
ENV APP_ENV production

ENTRYPOINT ["./zero2prod"]
