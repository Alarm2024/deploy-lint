FROM ubuntu:22.04
RUN useradd -m -u 10001 schedule
USER schedule
HEALTHCHECK --interval=30s --timeout=3s CMD curl -f http://127.0.0.1:8080/health || exit 1
CMD ["/usr/local/bin/schedule-app"]
