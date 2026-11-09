FROM node:20.11.0
USER node
HEALTHCHECK CMD curl -f http://localhost/ || exit 1
CMD ["node", "server.js"]
