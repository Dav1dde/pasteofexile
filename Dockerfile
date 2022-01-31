FROM node:current

WORKDIR /opt
RUN wget --quiet https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init \
	&& chmod +x rustup-init \
	&& ./rustup-init --quiet -y --profile minimal --no-modify-path \
	&& rm rustup-init
ENV PATH /root/.cargo/bin:$PATH
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk wasm-pack worker-build

WORKDIR /pasteofexile
EXPOSE 8787
