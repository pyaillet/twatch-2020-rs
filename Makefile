CUSTOM_RUSTC = /Users/pyaillet/Projets/esp32/rust-xtensa
DEVICE = /dev/cu.usbserial-0236B9E6
SPEED = 115200
RUST_BACKTRACE = 1 
XARGO_RUST_SRC = $(CUSTOM_RUSTC)/library # or /src for an older compiler
RUSTC = $(CUSTOM_RUSTC)/build/x86_64-apple-darwin/stage2/bin/rustc
RUSTDOC = $(CUSTOM_RUSTC)/build/x86_64-apple-darwin/stage2/bin/rustdoc
FEATURES = "xtensa-lx-rt/lx6,xtensa-lx/lx6,esp32,esp32-hal"

.PHONY: watch
watch: flash
	screen $(DEVICE) $(SPEED)

.PHONY: flash
flash: build
	cargo espflash --chip esp32 --speed $(SPEED) --features=$(FEATURES) $(DEVICE)

.PHONY: build
build:
	cargo xbuild --features=$(FEATURES)
