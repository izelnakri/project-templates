#!/usr/bin/env python3

# requires nix shell nixpkgs#python3Packages.pygobject3 nixpkgs#webkitgtk

import gi
gi.require_version('WebKit2', '4.0')
from gi.repository import Gtk, WebKit2

win = Gtk.Window(title="Nix Dependency Status")
view = WebKit2.WebView()
view.load_uri("file://" + os.path.abspath("data/index.html"))
win.add(view)
win.set_default_size(800, 600)
win.connect("destroy", Gtk.main_quit)
win.show_all()
Gtk.main()
