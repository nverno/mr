SHELL = /bin/bash

PROTO  =  $(CURDIR)/proto/rpc.proto

GODIR      =  $(CURDIR)/go
GOSRC      =  $(shell find $(GODIR) -type f -name \*.go)
GOAPPSDIR =  $(GODIR)/mrapps
GOAPPS     =  $(shell find $(GOAPPSDIR) -type f -name \*.go)
GOFLAGS    ?= -race

# export GO111MODULE=on

all:
	@

install:
	go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
	go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

gopb: install $(PROTO)
	protoc -I$(CURDIR) --go_out=$(CURDIR) \
	--go-grpc_out=$(CURDIR)               \
	$(PROTO)

gobuild: $(GOSRC)
	cd "$(GOAPPSDIR)" && go build $(GOFLAGS) -buildmode=plugin *

$(GOAPPSDIR)/%.so: $(GOAPPSDIR)/%.go
	cd "$(GOAPPSDIR)" && go build $(GOFLAGS) -buildmode=plugin $^

wc.so: $(GOAPPSDIR)/wc.so


clean:
	$(RM) -r core *~ *.o *.out *.exe

distclean: clean
	$(RM) $(GOAPPSDIR)/*.so *.so 
