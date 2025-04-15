# TODO: how to version the deps?
# Make them runnable with flatpak
project('github_user_fetcher', 'c', version : '1.0.0', default_options : ['warning_level=3'])

cc = meson.get_compiler('c')

deps = [
  dependency('jansson', required: true),
  dependency('libcurl', required: true),
]

incdir = include_directories('src')

sources = files('src/main.c', 'src/user.c')

# Build the executable
executable('github_user_fetcher', sources,
  dependencies: deps,
  include_directories: incdir,
  install: true
)

# Build static and shared libs
libsrc = files('src/user.c')
shared_library('github_user_fetcher', libsrc,
  dependencies: deps,
  include_directories: incdir,
  install: true
)

static_library('github_user_fetcher_static', libsrc,
  dependencies: deps,
  include_directories: incdir,
  install: true
)

# Install header file
install_headers('src/user.h', subdir: 'github_user_fetcher')
