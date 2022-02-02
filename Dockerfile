FROM node:current

# move HOME away from /root so we can run as unprivileged user
ENV HOME /tmp

WORKDIR /opt
RUN wget --quiet https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init \
	&& chmod +x rustup-init \
	&& ./rustup-init --quiet -y --profile minimal --no-modify-path \
	&& rm rustup-init
ENV PATH /tmp/.cargo/bin:$PATH
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk wasm-pack worker-build wrangler

# make world-writable so we can run as unprivileged user
RUN find /tmp/.cargo -print0 | xargs -0 chmod o+rw

WORKDIR /pasteofexile
EXPOSE 8787
