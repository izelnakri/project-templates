project('github_user_fetcher', 'c', 'cpp', version : '1.0.0')

cc = meson.get_compiler('c')
warning_flags = ['-Wall', '-Wextra']

dependencies = [
  dependency('jansson', required: true), # static: true makes it prefer x.a over x.so
  dependency('libcurl', required: true),
]

headers = include_directories('src', 'vendor/civetweb')

executable('github_user_fetcher', files('src/main.c', 'src/user.c', 'src/server.c', 'vendor/civetweb/civetweb.c'),
  dependencies: dependencies,
  include_directories: headers,
  c_args: warning_flags + ['-DOPENSSL_API_3_0'],
  install: true,
)

executable('github_user_fetcher_gui', files('src/gui.c'),
  dependencies: [
    # dependency('gtk+-3.0', required: true),
    dependency('gtk4', required: true),
    # dependency('webkitgtk-6.0', required: true)
    # dependency('webkit2gtk-4.1', required: true)
  ],
  c_args: warning_flags,
  install: true,
)

lib_src = files('src/user.c')

github_user_fetcher_shared = shared_library('github_user_fetcher', lib_src,
  dependencies: dependencies,
  include_directories: headers,
  c_args: warning_flags,
  install: true,
)

github_user_fetcher_static = static_library('github_user_fetcher_static', lib_src,
  dependencies: dependencies,
  include_directories: headers,
  c_args: warning_flags + ['-DOPENSSL_API_3_0'],
  install: true
)

install_headers('src/user.h', subdir: 'github_user_fetcher') # installs to /{prefix}/include/github_user_fetcher/user.h

tests_opt = get_option('tests')
build_tests = not tests_opt.disabled()

if build_tests
  test_dependencies = [
    dependency('criterion', required: true)
  ]

  test_executable = executable('test_user', 
    files('tests/test_user.c'),
    include_directories: headers,
    dependencies: dependencies + test_dependencies,
    link_with: github_user_fetcher_static,
    c_args: warning_flags + ['-DTESTING']
  )
  test('user_tests', test_executable)
endif

if get_option('benchmarks').enabled()
  benchmark_dependencies = [
    dependency('benchmark', required: true)
  ]

  benchmark_executable = executable('bench_user',
    files('benchmarks/bench_user.cpp'), # Maybe I need to add user.c here
    include_directories: headers,
    dependencies: dependencies + benchmark_dependencies,
    link_with: github_user_fetcher_static,
    c_args: warning_flags + ['-DBENCHMARKING']
  )
  test('user_benchmarks', benchmark_executable, is_parallel: false, suite: ['benchmark'])
endif

pkg = import('pkgconfig')
pkg.generate(
  name : 'github_user_fetcher',
  description : 'My github_user_fetcher library',
  libraries : github_user_fetcher_shared,
  version : '0.1.0')
