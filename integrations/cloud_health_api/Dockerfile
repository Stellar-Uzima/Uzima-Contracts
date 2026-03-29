FROM node:18-alpine AS builder
WORKDIR /app
COPY package.json package-lock.json* ./
RUN npm ci --only=production || npm install --only=production
COPY . .

FROM node:18-alpine
WORKDIR /app
COPY --from=builder /app .
ENV NODE_ENV=production
EXPOSE 3000
CMD ["node","src/index.js"]
