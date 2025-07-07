CC                  = clang
C_FLAGS             = -c -target wasm32-unknown-none -I$(INC_DIR)

DUMP_LAYOUTS_FLAGS  = -o /dev/null -emit-llvm -femit-all-decls -Xclang -fdump-record-layouts


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
SOURCES             = $(shell find $(SRC_DIR) -type f -name *.$(SRC_EXT))

WASM_FILES          = $(patsubst $(SRC_DIR)/%, $(TARGET_DIR)/%, $(SOURCES:.$(SRC_EXT)=.$(WASM_EXT)))
WAT_FILES           = $(patsubst $(SRC_DIR)/%, $(TARGET_DIR)/%, $(SOURCES:.$(SRC_EXT)=.$(WAT_EXT)))

SRC_LAYOUTS         = $(addsuffix .$(LAYOUT_EXT), $(patsubst $(SRC_DIR)/%, $(TARGET_DIR)/%, $(SOURCES)))
HEADER_LAYOUTS      = $(addsuffix .$(LAYOUT_EXT), $(patsubst $(INC_DIR)/%, $(TARGET_DIR)/%, $(HEADERS)))


.PHONY: all directories clean format layouts setup


all: directories $(WASM_FILES) $(WAT_FILES) layouts

directories:
	@mkdir -p -- $(SRC_DIR) $(TARGET_DIR) $(INC_DIR)

clean:
	@rm -rf -- $(TARGET_DIR) compile_commands.json

format:
	clang-format -i -- $(SOURCES)

layouts: $(SRC_LAYOUTS) $(HEADER_LAYOUTS)


# for legal reasons we cannot include the header files, but we can automate the acquisition :)
setup: $(INC_DIR)/ARINC653-wasm.h


#
### Rules magic
#

# rule to download the ARINC headerfiles
$(TARGET_DIR)/unprocessed-headers/ARINC653.h $(TARGET_DIR)/unprocessed-headers/ARINC653P2.h &:
	@mkdir -p -- $(TARGET_DIR)/{downloads,unprocessed-headers}
	# curl --location --output $(TARGET_DIR)/downloads/arinc653.h.zip https://brx-content.fullsight.org/site/binaries/content/assets/itc/content/support-files/arinc653.h.zip
	curl --user-agent 'Mozilla/5.0 (Windows NT 6.3; WOW64; rv:41.0) Gecko/20100101 Firefox/41.0' \
		--location --output-dir $(TARGET_DIR)/downloads/ --remote-name-all \
		https://brx-content.fullsight.org/site/binaries/content/assets/itc/content/support-files/arinc653{,p2}.h.zip
	echo $(TARGET_DIR)/downloads/*.zip | xargs --max-args=1 bsdtar -x --cd $(TARGET_DIR)/unprocessed-headers --file

# rule to generate our Wasm header file, by making every open type a 32 Bit integer
$(INC_DIR)/ARINC653-wasm.h: $(TARGET_DIR)/unprocessed-headers/ARINC653.h
	mkdir -p -- $(INC_DIR)
	sed 's/<an APEX integer type>/APEX_INTEGER/' $< > $@

# Rule to compile C to Wasm
$(TARGET_DIR)/%.$(WASM_EXT): $(SRC_DIR)/%.$(SRC_EXT) directories
	$(CC) $(C_FLAGS) -o$@ -- $<

# Rule to export Wasm Text (Wat) from Wasm)	
%.$(WAT_EXT): %.$(WASM_EXT) directories
	$(WASM2WAT) --output=$@ $(WASM2WAT_FLAGS) -- $<

# Rule to dump layout of data types in program
$(TARGET_DIR)/%.c.$(LAYOUT_EXT): $(SRC_DIR)/%.$(SRC_EXT) directories
	@: > $@
	@echo "-----------fdump-record-layouts" >> $@
	$(CC) $(C_FLAGS) $(DUMP_LAYOUTS_FLAGS) -- $< >> $@
	@echo "-----------fdump-record-layouts-canonical" >> $@
	$(CC) $(C_FLAGS) $(DUMP_LAYOUTS_FLAGS)-canonical -- $< >> $@

# Rule to dump layout of data types from header
$(TARGET_DIR)/%.h.$(LAYOUT_EXT): $(INC_DIR)/%.$(HEADER_EXT) directories
	@: > $@
	$(CC) $(C_FLAGS) $(DUMP_LAYOUTS_FLAGS)-complete -- $< >> $@
