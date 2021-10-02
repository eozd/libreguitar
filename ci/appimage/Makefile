default_prefix = /usr/local
prefix ?= $(default_prefix)
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
libdir = $(exec_prefix)/lib
includedir = $(prefix)/include
datarootdir = $(prefix)/share
datadir = $(datarootdir)

PROJECT_ROOT = ../..
CARGO_TOML = $(PROJECT_ROOT)/Cargo.toml
CARGO_LOCK = $(PROJECT_ROOT)/Cargo.lock
SOURCES = $(shell find $(PROJECT_ROOT)/src -type f -wholename "*src/*.rs") $(CARGO_TOML) $(CARGO_LOCK)

RELEASE = debug
DEBUG ?= 0
ifeq (0,$(DEBUG))
	ARGS = --release
	RELEASE = release
endif

VENDORED ?= 0
ifeq (1,$(VENDORED))
    ARGS += --frozen
endif

TARGET = $(PROJECT_ROOT)/target/$(RELEASE)

.PHONY: all clean distclean install uninstall update

BIN=libreguitar
APPID=com.github.eozd.libreguitar
APPDATA=$(APPID).appdata.xml
DESKTOP=$(APPID).desktop
ICON=48x48/apps/$(APPID).png

all: app

app: $(TARGET)/$(BIN) $(TARGET)/$(BIN).1.gz $(SOURCES)

clean:
	cargo clean

distclean: clean
	rm -rf .cargo vendor vendor.tar

vendor: vendor.tar

vendor.tar:
	mkdir -p .cargo
	cargo vendor | head -n -1 > .cargo/config
	echo 'directory = "vendor"' >> .cargo/config
	tar pcf vendor.tar vendor
	rm -rf vendor

install-app: app
	install -Dm 0755 "$(TARGET)/$(BIN)" "$(DESTDIR)$(bindir)/$(BIN)"
	install -Dm 0644 "$(TARGET)/$(BIN).1.gz" "$(DESTDIR)$(datadir)/man/man1/$(BIN).1.gz"
	install -Dm 0644 "$(DESKTOP)" "$(DESTDIR)$(datadir)/applications/$(DESKTOP)"
	install -Dm 0644 "$(ICON)" "$(DESTDIR)$(datadir)/icons/hicolor/$(ICON)"

install: all install-app

uninstall-app:
	rm -f "$(DESTDIR)$(bindir)/$(BIN)"
	rm -f "$(DESTDIR)$(datadir)/man/man1/$(BIN).1.gz"
	rm -f "$(DESTDIR)$(datadir)/applications/$(DESKTOP)"
	rm -f "$(DESTDIR)$(datadir)/icons/hicolor/$(ICON)"

uninstall: uninstall-app

update:
	cargo update

extract:
ifeq ($(VENDORED),1)
	tar pxf vendor.tar
endif

$(TARGET)/$(BIN): extract
	cargo build --manifest-path $(CARGO_TOML) $(ARGS)

$(TARGET)/$(BIN).1.gz: $(TARGET)/$(BIN)
	help2man --no-info $< | gzip -c > $@.partial
	mv $@.partial $@
