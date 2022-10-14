FROM  zhhanscn/builder:golang-1.19.2-alpine-3.16 as  builder
ARG COMPRESS='false'
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN go build -trimpath -a -ldflags "-linkmode external -w -s -extldflags  -static " -gcflags "-l=4 -m=2 "  --tags netgo -o dist/kpm kpm/cmd/kpm  \
    &&  if [  ${COMPRESS} = 'true' ]; then upx -9 dist/kpm; fi




FROM zhhanscn/runtime-env:alpine-3.16.2-git-2.36.2
COPY  --from=builder  /app/dist/ /app/
CMD ["/bin/sh"]

