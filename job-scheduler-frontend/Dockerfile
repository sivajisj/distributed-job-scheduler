# frontend/Dockerfile

# 1. Build Stage
FROM node:20-alpine AS builder

WORKDIR /app
COPY package.json package-lock.json ./
RUN npm install

COPY . .
RUN npm run build

# 2. Production Stage
FROM node:20-alpine
WORKDIR /app

# Set production environment variables
ENV NODE_ENV production
# Next.js will use this port
ENV PORT 3000

COPY --from=builder /app/.next ./.next
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/public ./public

# The Next.js production server command
CMD ["npm", "start"]