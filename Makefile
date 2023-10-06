SHELL     =  /bin/bash
AWK       ?= gawk

PROTO     =  $(CURDIR)/proto/rpc.proto
PROTOC    ?= protoc

DATADIR   = $(CURDIR)/data

GODIR     =  $(CURDIR)/go
GOSRC     =  $(shell find $(GODIR) -type f -name \*.go)
GOAPPSDIR =  $(GODIR)/mrapps
GOAPPS    =  $(shell find $(GOAPPSDIR) -type f -name \*.go)
GOFLAGS   ?= -race

RUSTDIR   = $(CURDIR)/rust

# export GO111MODULE=on

all:
	@

install:
	go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
	go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

gopb: install $(PROTO) ## Generate go protobuf code
	$(PROTOC) -I$(CURDIR) --go_out=$(CURDIR) \
		--go-grpc_out=$(CURDIR)          \
		$(PROTO)

# gobuild: $(GOSRC)
# 	cd "$(GOAPPSDIR)" && go build $(GOFLAGS) -buildmode=plugin *

$(GOAPPSDIR)/%.so: $(GOAPPSDIR)/%.go
	cd "$(GOAPPSDIR)" && go build $(GOFLAGS) -buildmode=plugin $^

wc.so: $(GOAPPSDIR)/wc.so  ## Build word count plugin for go impl.


.PHONY: test-go test-rust build-rust
build-rust: ## build rust mapreducer
	cd $(RUSTDIR) && cargo build --workspace

test-go: ## run go mapreducer using plugins
	cd $(GODIR)/main && ./run.sh -r

test-rust: ## run rust mapreducer using plugins
	cd $(RUSTDIR) && ./run.sh $(DATADIR)/pg-*.txt

clean:
	$(RM) -r core *~ *.o *.out *.exe

distclean: clean
	$(RM) $(GOAPPSDIR)/*.so *.so 


.PHONY: help
help:  ## Display this help message
	@for mfile in $(MAKEFILE_LIST); do                  \
	  grep -E '^[a-zA-Z_%-]+:.*?## .*$$' $$mfile |      \
	  sort | ${AWK}                                     \
	  'BEGIN {FS = ":.*?## "};                          \
	   {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'; \
	done
