FROM mcr.microsoft.com/azurelinux/base/rust:1

COPY target/release/chick /usr/local/bin/chick

ENTRYPOINT ["chick"]
