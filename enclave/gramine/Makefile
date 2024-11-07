# ARCH LINUX
#ARCH_LIBDIR ?= /lib/$(shell $(CC) -dumpmachine)

# UBUNTU LINUX
ARCH_LIBDIR ?= /lib/x86_64-linux-gnu

# NOTE: MUST BE "RELATIVE PATH" TO TEE BINARY
ENCLAVEDIR ?= "."
SELF_EXE = $(ENCLAVEDIR)/bin/tee_ai_agent

SGX ?= 1
DEBUG ?= 0
SGX_DEV_BUILD ?= 1

.PHONY: all
all: tee_ai_agent.manifest
ifeq ($(SGX),1)
all: tee_ai_agent.manifest.sgx tee_ai_agent.sig
endif

ifeq ($(DEBUG),2)
GRAMINE_LOG_LEVEL = debug
else ifeq ($(DEBUG),1)
GRAMINE_LOG_LEVEL = warning
else
GRAMINE_LOG_LEVEL = error
endif

ifeq ($(SGX_DEV_BUILD),1)
MANIFEST_FILE = dev_tee_ai_agent.manifest.template
else
MANIFEST_FILE = tee_ai_agent.manifest.template
endif

tee_ai_agent.manifest: $(MANIFEST_FILE)
	gramine-manifest \
		-Dlog_level=$(GRAMINE_LOG_LEVEL) \
		-Darch_libdir=$(ARCH_LIBDIR) \
		-Dself_exe=$(SELF_EXE) \
		-Denclave_dir=$(ENCLAVEDIR) \
		$< $@

tee_ai_agent.manifest.sgx tee_ai_agent.sig &: tee_ai_agent.manifest
	gramine-sgx-sign -d today \
		--manifest $< \
		--output $<.sgx

ifeq ($(SGX),)
GRAMINE = gramine-direct
else
GRAMINE = gramine-sgx
endif 

.PHONY: start-gramine-server
start-gramine-server: all
	SGX_AESM_ADDR=1 $(GRAMINE) tee_ai_agent \
		--domain $(SGX_DOMAIN) \
		--port $(SGX_PORT) \
		--verbose $(SGX_VERBOSITY) #>> $(ENCLAVEDIR)/enclave.log 2>&1 &

.PHONY: clean
clean:
	$(RM) -rf $(ENCLAVEDIR)/*.token $(ENCLAVEDIR)/*.sig $(ENCLAVEDIR)/*.manifest.sgx $(ENCLAVEDIR/)*.manifest $(ENCLAVEDIR)/result-* $(ENCLAVEDIR)/*.log

.PHONY: distclean
distclean: clean
	$(RM) -rf target/ Cargo.lock
