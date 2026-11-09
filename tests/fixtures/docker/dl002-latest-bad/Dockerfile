FROM node:latest
USER node
HEALTHCHECK CMD curl -f http://localhost/ || exit 1
CMD ["node", "server.js"]
