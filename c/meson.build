project('github_user_fetcher', 'c', version : '1.0.0', default_options : ['warning_level=3'])

cc = meson.get_compiler('c')

dependencies = [
  dependency('jansson', required: true),
  dependency('libcurl', required: true),
]

headers = include_directories('src', 'vendor/civetweb')
executable_src = files('src/main.c', 'src/user.c', 'src/server.c', 'vendor/civetweb/civetweb.c')

executable('github_user_fetcher', executable_src,
  dependencies: dependencies,
  include_directories: headers,
  c_args: ['-DOPENSSL_API_3_0'],
  install: true,
)

lib_src = files('src/user.c')
shared_library('github_user_fetcher', lib_src,
  dependencies: dependencies,
  include_directories: headers,
  install: true,
)

static_library('github_user_fetcher_static', lib_src,
  dependencies: dependencies,
  include_directories: headers,
  c_args: ['-DOPENSSL_API_3_0'],
  install: true
)

install_headers('src/user.h', subdir: 'github_user_fetcher') # installs to /{prefix}/include/github_user_fetcher/user.h
