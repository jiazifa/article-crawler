FROM node:latest

COPY . /app

WORKDIR /app

RUN pnpm start

RUN pnpm install --frozen-lockfile --only=prod

EXPOSE 3000

CMD [ "pnpm", "start" ]