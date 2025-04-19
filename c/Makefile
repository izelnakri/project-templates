CC = gcc
AR = ar

PKGS = jansson libcurl
CFLAGS += -Wall -std=c11 -fPIC -g -I./src $(shell pkg-config --cflags $(PKGS))
LDFLAGS += $(shell pkg-config --libs-only-L $(PKGS))
LIBS = $(shell pkg-config --libs-only-l $(PKGS))

SRC = src/main.c src/user.c
LIBSRC = src/user.c

OBJ = $(SRC:.c=.o)
LIBOBJ = $(LIBSRC:.c=.o)

TARGET = github_user_fetcher
STATIC_LIB = libgithub_user_fetcher.a
SHARED_LIB = libgithub_user_fetcher.so

PREFIX ?= /usr
LIBDIR ?= $(PREFIX)/lib
INCLUDEDIR ?= $(PREFIX)/include

.PHONY: all clean install debug

all: $(TARGET) $(STATIC_LIB) $(SHARED_LIB)

$(TARGET): $(OBJ)
	$(CC) -o $@ $(OBJ) $(LDFLAGS) $(LIBS)

$(STATIC_LIB): $(LIBOBJ)
	$(AR) rcs $@ $^

$(SHARED_LIB): $(LIBOBJ)
	$(CC) -shared -o $@ $^ $(LDFLAGS) $(LIBS)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

install: all
	install -D $(TARGET) $(DESTDIR)$(PREFIX)/bin/$(TARGET)
	install -D $(STATIC_LIB) $(DESTDIR)$(LIBDIR)/$(STATIC_LIB)
	install -D $(SHARED_LIB) $(DESTDIR)$(LIBDIR)/$(SHARED_LIB)
	install -D src/user.h $(DESTDIR)$(INCLUDEDIR)/github_user_fetcher/user.h

prepare-flatpak-builder:
	sh flatpak/prepare-nix-deps.sh
flatpak-builder: 
	flatpak-builder flatpak/build flatpak/org.example.GithubUserFetcher.json --force-clean
build-flatpak: clean-flatpak-build prepare-flatpak-builder flatpak-builder
run-flatpak-cli: build-flatpak
	echo "Running org.example.GithubUserFetcher.json CLI:"
	flatpak-builder --run flatpak/build flatpak/org.example.GithubUserFetcher.json github_user_fetcher
run-flatpak-gui: build-flatpak
	echo "Running org.example.GithubUserFetcher.json GUI:"
	flatpak-builder --run flatpak/build flatpak/org.example.GithubUserFetcher.json github_user_fetcher_gui
run-flatpak: run-flatpak-gui
clean-flatpak-build:
	rm -rf .flatpak-builder flatpak/build flatpak/nix-deps.tar.gz

test:
	meson compile -C build && meson test -C build

clean:
	rm -f $(OBJ) $(LIBOBJ) $(TARGET) $(STATIC_LIB) $(SHARED_LIB)

debug:
	@echo "CFLAGS: $(CFLAGS)"
	@echo "LDFLAGS: $(LDFLAGS)"
	@echo "LIBS: $(LIBS)"
