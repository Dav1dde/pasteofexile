FROM rust:1.58

WORKDIR /pasteofexile
RUN cargo install --root /usr/local -- trunk wasm-pack worker-build wrangler

FROM rust:1.58
COPY --from=0 /usr/local/bin/* /usr/local/bin/

RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash -

RUN apt-get update -y && apt-get install -y nodejs && rm -rf /var/lib/apt/lists/*

RUN curl -sL https://dl.yarnpkg.com/debian/pubkey.gpg | gpg --dearmor > /usr/share/keyrings/yarnkey.gpg \
    && echo "deb [signed-by=/usr/share/keyrings/yarnkey.gpg] https://dl.yarnpkg.com/debian stable main" > /etc/apt/sources.list.d/yarn.list \
    && apt-get update -y \
    && apt-get install -y yarn \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown

ENV HOME /tmp

WORKDIR /pasteofexile
