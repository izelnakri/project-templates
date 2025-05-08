project('github_user_fetcher', 'cpp', version : '1.0.0')

cpp = meson.get_compiler('cpp')
warning_flags = ['-Wall', '-Wextra', '-std=c++20'] # TODO: in future move it to c++23(+)

dependencies = [
  dependency('boost', modules : ['system'], required : true),
  dependency('openssl', required: true),
  dependency('nlohmann_json', required: true),
]

# NOTE: Old one: headers = include_directories('src', 'vendor/civetweb')
headers = include_directories('src')

executable('github_user_fetcher', files('src/main.cpp', 'src/user.cpp', 'src/server.cpp'),
  dependencies: dependencies,
  include_directories: headers,
  cpp_args: warning_flags,
  install: true,
)

lib_src = files('src/user.cpp')

github_user_fetcher_shared = shared_library('github_user_fetcher', lib_src,
  dependencies: dependencies,
  include_directories: headers,
  cpp_args: warning_flags,
  install: true,
)

github_user_fetcher_static = static_library('github_user_fetcher_static', lib_src,
  dependencies: dependencies,
  include_directories: headers,
  cpp_args: warning_flags,
  install: true
)

install_headers('src/user.hpp', subdir: 'github_user_fetcher') # ?!Does it instal to /{prefix}/include/github_user_fetcher/user

css_header = custom_target('style_css',
  input: 'src/style.css',
  output: 'style_css.hpp',
  command: [
    'sh', '-c',
    'echo "#pragma once\n\ninline const char* css_data = \"$(cat @INPUT@ | sed \'s/\"/\\\\\"/g\')\";" > @OUTPUT@'
  ]
)

configure_file(input: 'src/style.css', output: 'style.css')
custom_target('copy_style_css',
  command: ['cp', 'src/style.css', join_paths(get_option('builddir'), 'style.css')],
  output: 'style.css'
)

executable('github_user_fetcher_gui', ['src/gui.cpp'],
  dependencies: [
    dependency('gtkmm-4.0', required: true),
    # In future: If using webkit, add dependencies here.
  ],
  include_directories: headers,
  link_with: github_user_fetcher_static,
  cpp_args: warning_flags,
  install: true,
)

install_data('src/style.css', install_dir: 'share/github_user_fetcher')

# Testing setup
# tests_opt = get_option('tests')
# build_tests = not tests_opt.disabled()

# if build_tests
#   test_dependencies = [
#     dependency('catch2', required: true),  # Using Catch2 for C++ tests
#   ]

#   test_executable = executable('test_user', 
#     files('tests/test_user.cpp'),
#     include_directories: headers,
#     dependencies: [boost_dep, nlohmann_json_dep] + test_dependencies,  # Added nlohmann_json dependency
#     link_with: github_user_fetcher_static,
#     cpp_args: warning_flags + ['-DTESTING']
#   )
#   test('user_tests', test_executable)
# endif

# Benchmark setup
# if get_option('benchmarks').enabled()
#   benchmark_dependencies = [
#     dependency('benchmark', required: true),
#   ]
#
#   benchmark_executable = executable('bench_user',
#     files('benchmarks/bench_user.cpp'), 
#     include_directories: headers,
#     dependencies: [boost_dep, nlohmann_json_dep] + benchmark_dependencies,  # Added nlohmann_json dependency
#     link_with: github_user_fetcher_static,
#     cpp_args: warning_flags + ['-DBENCHMARKING']
#   )
#   test('user_benchmarks', benchmark_executable, is_parallel: false, suite: ['benchmark'])
# endif

pkg = import('pkgconfig')
pkg.generate(
  name : 'github_user_fetcher',
  description : 'My github_user_fetcher library',
  libraries : github_user_fetcher_shared,
  version : '0.1.0'
)
