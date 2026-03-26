FROM ubuntu:24.04

RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    libc6-dev \
    make \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Clone selfie (includes C* compiler, rotor, and benchmark programs)
RUN git clone --depth 1 https://github.com/cksystemsteaching/selfie.git /selfie

WORKDIR /selfie

COPY compile.sh /compile.sh
RUN chmod +x /compile.sh

ENTRYPOINT ["/compile.sh"]
