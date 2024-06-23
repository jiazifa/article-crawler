FROM python:3.11-slim AS base

# 2. Rebuild the source code only when needed
FROM base AS builder

RUN apt-get update && apt-get install -y --no-install-recommends build-essential gcc

WORKDIR /app

RUN python -m venv /opt/venv

ENV PATH="/opt/venv/bin:$PATH"

COPY ./requirements.lock ./requirements.lock

RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir --upgrade setuptools wheel && \
    pip install --no-cache-dir --upgrade -r requirements.lock

FROM base AS runner
COPY --from=builder /opt/venv /opt/venv

WORKDIR /app

# Make sure we use the virtualenv:
ENV PATH="/opt/venv/bin:$PATH"

COPY . .

# copy the built files from the previous stage

