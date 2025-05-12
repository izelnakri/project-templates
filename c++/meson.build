project('github_user_fetcher', 'cpp', version : '1.0.0')

cpp = meson.get_compiler('cpp')
cpp_args = ['-Wall', '-Wextra', '-std=c++20'] # TODO: in future move it to c++23(+)
lib_src = files('src/user.cpp', 'src/server.cpp')
main_src = lib_src + files('src/main.cpp', 'src/config.cpp')

# NOTE: Old one: headers = include_directories('src', 'vendor/civetweb')
headers = include_directories('src')

dependencies = [
  dependency('boost', modules : ['system'], required : true),
  dependency('openssl', required: true),
  dependency('nlohmann_json', required: true),
]

github_user_fetcher = executable('github_user_fetcher', main_src,
  dependencies: dependencies,
  include_directories: headers,
  cpp_args: cpp_args,
  install: true,
)

github_user_fetcher_shared = shared_library('github_user_fetcher', lib_src,
  dependencies: dependencies,
  include_directories: headers,
  cpp_args: cpp_args,
  install: true,
)

github_user_fetcher_static = static_library('github_user_fetcher_static', lib_src,
  dependencies: dependencies,
  include_directories: headers,
  cpp_args: cpp_args,
  install: true
)

install_headers('src/user.hpp', subdir: 'github_user_fetcher') # ?!Does it install to /{prefix}/include/github_user_fetcher/user

css_header = custom_target('style_css_hpp', # Generate header from CSS file
  input: 'src/style.css',
  build_always_stale: true,
  build_by_default: true,
  output: 'style_css.hpp',
  command: [
    'sh', '-c',
    '''
    {
      echo '#pragma once'
      echo ''
      echo 'inline const char* css_data = R"CSS_CONTENT('
      cat @INPUT@
      echo ')CSS_CONTENT";'
    } > "$2"
    ''', 'sh', '@INPUT@', '@OUTPUT@'
  ]
)

executable('github_user_fetcher_gui',
  ['src/gui.cpp', css_header],
  dependencies: [
    dependency('gtkmm-4.0', required: true),
    # In future: If using webkit, add dependencies here.
  ],
  include_directories: headers,
  link_with: github_user_fetcher_static,
  cpp_args: cpp_args,
  install: true,
)

install_data('src/style.css', install_dir: 'share/github_user_fetcher') # NOTE: maybe change the code to load this dynamically in future

# NOTE: Below made redundant:
# configure_file(input: 'src/style.css', output: 'style.css')
# custom_target('copy_style_css',
#   command: ['cp', 'src/style.css', join_paths(get_option('builddir'), 'style.css')],
#   output: 'style.css'
# )

tests_opt = get_option('tests')
build_tests = not tests_opt.disabled()

if build_tests
  test_dependencies = [
    dependency('catch2-with-main', required: true),
    dependency('libcurl', required: true),
  ]

  test_sources = {
    'unit': files('tests/unit/test_user.cpp', 'tests/unit/test_main.cpp' ),
    'api': files('tests/api/test_server.cpp'),
    'cli': files('tests/cli/test_cli.cpp'),
  }

  foreach suite, sources : test_sources
    test_exe = executable(
      'test_' + suite,
      sources,
      include_directories: [headers],
      dependencies: dependencies + test_dependencies,
      link_with: github_user_fetcher_static,
      cpp_args: cpp_args
    )

    test(suite + '_tests', test_exe, suite: suite, depends: suite == 'cli' ? [github_user_fetcher] : [])
  endforeach
endif

if get_option('benchmarks').enabled()
  benchmark_src = files('benchmarks/bench_user.cpp')

  benchmark_dependencies = [
    dependency('benchmark', required: true), # or: # dependency('catch2-with-main', required: true),
  ]

  benchmark_executable = executable('benchmarks',
    benchmark_src,
    include_directories: headers,
    dependencies: dependencies + benchmark_dependencies,
    link_with: github_user_fetcher_static,
    cpp_args: cpp_args
  )

  test('benchmarks', benchmark_executable, is_parallel: false, suite: ['benchmark'])
endif

pkg = import('pkgconfig')
pkg.generate(
  name : 'github_user_fetcher',
  description : 'My github_user_fetcher library',
  libraries : github_user_fetcher_shared,
  version : '0.1.0'
)
