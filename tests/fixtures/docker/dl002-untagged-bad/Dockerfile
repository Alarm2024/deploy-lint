FROM nginx
USER nginx
HEALTHCHECK CMD curl -f http://localhost/ || exit 1
CMD ["nginx", "-g", "daemon off;"]
