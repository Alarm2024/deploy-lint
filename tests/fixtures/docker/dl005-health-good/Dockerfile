FROM alpine:3.19
USER nobody
HEALTHCHECK CMD wget -qO- http://127.0.0.1/ || exit 1
CMD ["/bin/sh"]
