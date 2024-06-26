FROM node:alpine

WORKDIR /app

RUN corepack enable && corepack prepare pnpm@latest --activate

COPY package.json pnpm-lock.yaml* ./

RUN pnpm fetch

COPY . /app

RUN pnpm install --frozen-lockfile --only=prod

EXPOSE 3000

CMD [ "pnpm", "start" ]