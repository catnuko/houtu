GLSLC = $(shell which glslc)
ifeq "$(GLSLC)" ""
	break;
endif

FLAGS = -c -g

SHADERS=$(wildcard shader/**/*)
COMP_SHADERS = $(patsubst shader/%,compiled_shader/%.spv,$(SHADERS))
COMP_DISASMS = $(patsubst shader/%,compiled_shader/%.spvasm,$(SHADERS))

all: $(COMP_SHADERS) $(COMP_DISASMS)

compiled_shader/%.spv: shader/%
	mkdir -p $(dir $@)
	$(GLSLC) -MD -g -c -O -o $@ $<

compiled_shader/%.spvasm: shader/%
	mkdir -p $(dir $@)
	$(GLSLC) -MD -S -g -O -o $@ $<

clean:
	rm compiled_shader/**/*.spv compiled_shader/**/*.spvasm compiled_shader/**/*.d


PHONY: all clean