FROM node:20-alpine
WORKDIR /app
COPY package*.json ./
# Install dependencies for the entire monorepo
RUN npm install --omit=dev
COPY . .
ENV PORT=3000
ENV NODE_ENV=production
EXPOSE 3000
CMD ["node", "apps/docker-plus/src/index.js"]
