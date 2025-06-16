APP_NAME := content_provider
FLATPAK_ID := org.example.ContentProvider
SESSION_BUS_ADDRESS := $(shell echo $$DBUS_SESSION_BUS_ADDRESS)

.PHONY: all build run test flatpak-run

all: build

build:
	cargo build

run: export RUST_LOG=debug
run:
	cargo run

test:
	cargo test

flatpak-build:
	flatpak-builder flatpak/build flatpak/manifest.yml --force-clean
flatpak-run:
	flatpak-builder --run flatpak/build flatpak/manifest.yml env PATH=/usr/lib/sdk/rust-stable/bin:$PATH cargo run
# check to see if this is working:
flatpak-test:
	flatpak-builder --run flatpak/build flatpak/manifest.yml env PATH=/usr/lib/sdk/rust-stable/bin:$PATH cargo test 

get-dbus-xml-interface:
	busctl --user --xml-interface introspect \
		org.freedesktop.Notifications \
		/org/freedesktop/Notifications
generate-rust-proxy-dbus-interface:
	busctl --user --xml-interface introspect \
		org.freedesktop.Notifications \
		/org/freedesktop/Notifications > last-generate-dbus-interface.xml
	zbus-xmlgen file last-generate-dbus-interface.xml
dbus-introspect:
	busctl --user introspect org.freedesktop.secrets /org/freedesktop/secrets org.freedesktop.Secret.Service # last one is optional [interface]
call-dbus-interface:
	busctl --user call org.zbus.MyGreeter /org/zbus/MyGreeter org.zbus.MyGreeter1 SayHello s "zbus
	# $ busctl --user call org.zbus.MyGreeter /org/zbus/MyGreeter org.zbus.MyGreeter1 SayHello s "zbus"
	# s "Hello zbus!"

# $ busctl --user introspect org.zbus.MyGreeter /org/zbus/MyGreeter
# NAME                                TYPE      SIGNATURE RESULT/VALUE FLAGS
# [...]
# org.zbus.MyGreeter1                 interface -         -             -
# .GoAway                             method    -         -             -
# .SayHello                           method    s         s             -
# .GreeterName                        property  s         "GreeterName" emits-change writable
# .GreetedEveryone                    signal    -         -             -




# I can maybe add --command and make the cargo build --release via env injection on the flatpak-builder command

# flatpak-run:
# 	flatpak run --talk-name=org.freedesktop.portal.Clipboard \
# 	            --talk-name=org.freedesktop.portal.Contacts \
# 	            --filesystem=xdg-documents \
# 	            --own-name=$(FLATPAK_ID) \
# 	            --env=RUST_BACKTRACE=1 \
# 	            --env=RUST_LOG=debug \
# 	            --env=DBUS_SESSION_BUS_ADDRESS="$(SESSION_BUS_ADDRESS)" \
# 	            --command=sh \
# 	            org.freedesktop.Sdk//23.08 -c "cargo run"
#
# flatpak-test:
# 	flatpak run --talk-name=org.freedesktop.portal.Clipboard \
# 	            --talk-name=org.freedesktop.portal.Contacts \
# 	            --filesystem=xdg-documents \
# 	            --own-name=$(FLATPAK_ID) \
# 	            --env=RUST_BACKTRACE=1 \
# 	            --env=RUST_LOG=debug \
# 	            --env=DBUS_SESSION_BUS_ADDRESS="$(SESSION_BUS_ADDRESS)" \
# 	            --command=sh \
# 	            org.freedesktop.Sdk//23.08 -c "cargo test"
