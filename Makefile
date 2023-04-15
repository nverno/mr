SHELL = /bin/bash

PROTO = $(CURDIR)/proto/rpc.proto

all:
	@

gopb: $(PROTO)
	protoc -I$(CURDIR) --go_out=$(CURDIR) \
	--go-grpc_out=$(CURDIR)               \
	$(PROTO)
