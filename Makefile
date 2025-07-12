CC                  = clang
C_FLAGS             = --target=wasm32-unknown-wasi -I$(INC_DIR) -lc -nostartfiles
C_FLAGS_DEBUG       = $(C_FLAGS) -g
C_FLAGS_RELEASE     = $(C_FLAGS) -O3

LD_FLAGS            = --no-entry --export=cold_start --export=warm_start --allow-undefined-file=$(TARGET_DIR)/allow-undefined.syms --unresolved-symbols=report-all
EMPTY              :=
COMMA              := ,
C_FLAGS            += -Wl,$(subst $(EMPTY) $(EMPTY),$(COMMA),$(LD_FLAGS))

DUMP_LAYOUTS_FLAGS  = -c -o /dev/null -emit-llvm -femit-all-decls -Xclang -fdump-record-layouts


WASM2WAT            = wasm2wat
WASM2WAT_FLAGS      = --fold-exprs --enable-annotations --enable-code-metadata

TARGET_DIR          = target

INC_DIR             = include
HEADER_EXT          = h
SRC_DIR             = src
SRC_EXT             = c
WASM_EXT            = wasm
WAT_EXT             = wat
LAYOUT_EXT          = layout.txt


HEADERS             = $(shell find $(INC_DIR) -type f -name *.$(HEADER_EXT))
GENERATED_HEADERS   = $(INC_DIR)/ARINC653-wasm.h
SOURCES             = $(shell find $(SRC_DIR) -type f -name *.$(SRC_EXT))

WASM_FILES_DEBUG    = $(patsubst $(SRC_DIR)/%, $(TARGET_DIR)/debug/%, $(SOURCES:.$(SRC_EXT)=.$(WASM_EXT)))
WASM_FILES_RELEASE  = $(patsubst $(SRC_DIR)/%, $(TARGET_DIR)/release/%, $(SOURCES:.$(SRC_EXT)=.$(WASM_EXT)))
WAT_FILES           = $(WASM_FILES_DEBUG:.$(WASM_EXT)=.$(WAT_EXT)) $(WASM_FILES_RELEASE:.$(WASM_EXT)=.$(WAT_EXT))

SRC_LAYOUTS         = $(addsuffix .$(LAYOUT_EXT), $(patsubst $(SRC_DIR)/%, $(TARGET_DIR)/layouts/%, $(SOURCES)))
HEADER_LAYOUTS      = $(addsuffix .$(LAYOUT_EXT), $(patsubst $(INC_DIR)/%, $(TARGET_DIR)/layouts/%, $(HEADERS)))

ALL_TARGET_FILES    = $(WASM_FILES_DEBUG) $(WASM_FILES_RELEASE) $(WAT_FILES) $(SRC_LAYOUTS) $(HEADER_LAYOUTS) compile_commands.json

COMPILE_REQUISITES  = $(GENERATED_HEADERS) $(TARGET_DIR)/allow-undefined.syms

.PHONY: all clean clean-all format layouts setup


all: $(ALL_TARGET_FILES)

clean:
	@rm -f -- $(ALL_TARGET_FILES) compile_commands.json

clean-all:
	@rm -rf -- $(TARGET_DIR) compile_commands.json

format:
	clang-format -i -- $(SOURCES)

layouts: $(SRC_LAYOUTS) $(HEADER_LAYOUTS)


# for legal reasons we cannot include the header files, but we can automate the acquisition :)
setup: $(GENERATED_HEADERS)


#
### Rules magic
#

# rule to download the ARINC headerfiles
$(TARGET_DIR)/unprocessed-headers/ARINC653.h $(TARGET_DIR)/unprocessed-headers/ARINC653P2.h &:
	@mkdir -p -- $(@D) $(TARGET_DIR)/downloads
	curl --user-agent 'Mozilla/5.0 (Windows NT 6.3; WOW64; rv:41.0) Gecko/20100101 Firefox/41.0' \
		--location --output-dir $(TARGET_DIR)/downloads/ --remote-name-all \
		https://brx-content.fullsight.org/site/binaries/content/assets/itc/content/support-files/arinc653{,p2}.h.zip
	echo $(TARGET_DIR)/downloads/*.zip | xargs --max-args=1 bsdtar -x --cd $(TARGET_DIR)/unprocessed-headers --file

# rule to generate our Wasm header file, by making every open type a 32 Bit integer
$(INC_DIR)/ARINC653-wasm.$(HEADER_EXT) : $(TARGET_DIR)/unprocessed-headers/ARINC653.$(HEADER_EXT)
	mkdir -p -- $(@D) $(TARGET_DIR)
	sed 's/<an APEX integer type>/APEX_LONG_INTEGER/' $< > $@

# rule to list symbols which are allowed to be undefined
$(TARGET_DIR)/allow-undefined.syms : $(GENERATED_HEADERS)
	awk '$$1 == "extern" && $$2 == "void" {print $$3}' $^ > $@

# rule to compile C to Wasm in debug mode
$(TARGET_DIR)/debug/%.$(WASM_EXT) : $(SRC_DIR)/%.$(SRC_EXT) $(COMPILE_REQUISITES)
	@mkdir -p -- $(@D)/cdb-fragments
	$(CC) $(C_FLAGS_DEBUG) -gen-cdb-fragment-path $(@D)/cdb-fragments -o$@ -- $<

# rule to compile C to Wasm in release mode
$(TARGET_DIR)/release/%.$(WASM_EXT) : $(SRC_DIR)/%.$(SRC_EXT) $(COMPILE_REQUISITES)
	@mkdir -p -- $(@D)/cdb-fragments
	$(CC) $(C_FLAGS_RELEASE) -gen-cdb-fragment-path $(@D)/cdb-fragments -o$@ -- $<

# rule to concatenate a compile_commands.json
compile_commands.json: $(WASM_FILES_DEBUG) $(WASM_FILES_RELEASE)
	@echo '[' > $@
	@cat $(TARGET_DIR)/*/cdb-fragments/*.json >> $@
	@echo ']' >> $@

# rule to export Wasm Text (Wat) from Wasm
%.$(WAT_EXT) : %.$(WASM_EXT)
	$(WASM2WAT) --output=$@ $(WASM2WAT_FLAGS) -- $<

# rule to dump layout of data types in program
$(TARGET_DIR)/layouts/%.c.$(LAYOUT_EXT) : $(SRC_DIR)/%.$(SRC_EXT)
	@mkdir -p -- $(@D)
	@rm -f -- $@
	@echo "-----------fdump-record-layouts" >> $@
	$(CC) $(C_FLAGS) $(DUMP_LAYOUTS_FLAGS) -- $< >> $@
	@echo "-----------fdump-record-layouts-canonical" >> $@
	$(CC) $(C_FLAGS) $(DUMP_LAYOUTS_FLAGS)-canonical -- $< >> $@
	@echo "-----------fdump-record-layouts-complete" >> $@
	$(CC) $(C_FLAGS) $(DUMP_LAYOUTS_FLAGS)-complete -- $< >> $@

# rule to dump layout of data types from header
$(TARGET_DIR)/layouts/%.h.$(LAYOUT_EXT) : $(INC_DIR)/%.$(HEADER_EXT)
	@mkdir -p -- $(@D)
	@: > $@
	$(CC) -Xclang -disable-llvm-optzns $(C_FLAGS) $(DUMP_LAYOUTS_FLAGS) -- $< >> $@
