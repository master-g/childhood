#!/usr/bin/env bash
#target=mgnescli
target=pure6502
# format
echo "==> Formatting..."
find .. -type f -name '*.go' -not -path '../vendor/*' -print0 | xargs -0 goimports -w

# mod
echo "==> Module tidy and vendor..."
go mod tidy
go mod vendor
go mod download

# lint
echo "==> Linting..."
golangci-lint run ../...

# build
echo "==> Building $target"

go build -o ../bin/"${target}" ../cmd/"${target}"
