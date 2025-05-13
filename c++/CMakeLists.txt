cmake_minimum_required(VERSION 3.20)
project(github_user_fetcher VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE "Debug" CACHE STRING "Choose the type of build" FORCE)
    set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS "Debug" "Release" "RelWithDebInfo" "MinSizeRel")
endif()

add_compile_options(-Wall -Wextra)

set(LIB_SOURCES
    src/user.cpp
    src/server.cpp
)

set(MAIN_SOURCES
    src/main.cpp
    src/config.cpp
)

include_directories(src)

if(POLICY CMP0167)
    cmake_policy(SET CMP0167 OLD)
endif()

find_package(PkgConfig REQUIRED)
find_package(Boost COMPONENTS system)
if(NOT Boost_FOUND)
    pkg_check_modules(BOOST REQUIRED IMPORTED_TARGET boost)
    add_library(Boost::system ALIAS PkgConfig::BOOST)
endif()

find_package(OpenSSL REQUIRED) # NOTE: doesnt check from pkg-config most likely

find_package(nlohmann_json QUIET)
if(NOT nlohmann_json_FOUND)
    pkg_check_modules(NLOHMANN_JSON REQUIRED IMPORTED_TARGET nlohmann_json)
    add_library(nlohmann_json::nlohmann_json ALIAS PkgConfig::NLOHMANN_JSON)
endif()

set(DEPENDENCIES
    Boost::system
    OpenSSL::SSL
    OpenSSL::Crypto
    nlohmann_json::nlohmann_json
)

add_library(github_user_fetcher_static STATIC ${LIB_SOURCES})
target_include_directories(github_user_fetcher_static PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/src)
target_link_libraries(github_user_fetcher_static PUBLIC ${DEPENDENCIES})
set_target_properties(github_user_fetcher_static PROPERTIES OUTPUT_NAME github_user_fetcher)

add_library(github_user_fetcher_shared SHARED ${LIB_SOURCES})
target_include_directories(github_user_fetcher_shared PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/src)
target_link_libraries(github_user_fetcher_shared PUBLIC ${DEPENDENCIES})
set_target_properties(github_user_fetcher_shared PROPERTIES 
    OUTPUT_NAME github_user_fetcher
    VERSION ${PROJECT_VERSION}
    SOVERSION ${PROJECT_VERSION_MAJOR})

add_executable(github_user_fetcher ${MAIN_SOURCES})
target_link_libraries(github_user_fetcher PRIVATE github_user_fetcher_static)

add_custom_command(
    OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/style_css.hpp
    COMMAND ${CMAKE_COMMAND} -E echo "#pragma once" > ${CMAKE_CURRENT_BINARY_DIR}/style_css.hpp
    COMMAND ${CMAKE_COMMAND} -E echo "" >> ${CMAKE_CURRENT_BINARY_DIR}/style_css.hpp
    COMMAND ${CMAKE_COMMAND} -E echo "inline const char* css_data = R\"CSS_CONTENT(" >> ${CMAKE_CURRENT_BINARY_DIR}/style_css.hpp
    COMMAND ${CMAKE_COMMAND} -E cat ${CMAKE_CURRENT_SOURCE_DIR}/src/style.css >> ${CMAKE_CURRENT_BINARY_DIR}/style_css.hpp
    COMMAND ${CMAKE_COMMAND} -E echo ")CSS_CONTENT\";" >> ${CMAKE_CURRENT_BINARY_DIR}/style_css.hpp
    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/src/style.css
    VERBATIM
)
add_custom_target(css_header DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/style_css.hpp)

pkg_check_modules(GTKMM REQUIRED IMPORTED_TARGET gtkmm-4.0) # NOTE: naming of the first parameter is optional
add_library(gtkmm ALIAS PkgConfig::GTKMM) # NOTE: this eventually results in -lgtkmm, 

set(GUI_DEPENDENCIES
  gtkmm
)

add_executable(github_user_fetcher_gui src/gui.cpp ${CMAKE_CURRENT_BINARY_DIR}/style_css.hpp)
add_dependencies(github_user_fetcher_gui css_header)
target_include_directories(github_user_fetcher_gui PRIVATE ${CMAKE_CURRENT_BINARY_DIR})
target_link_libraries(github_user_fetcher_gui PRIVATE github_user_fetcher_static ${GUI_DEPENDENCIES})

file(GENERATE OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/github_user_fetcher.pc CONTENT 
"prefix=${CMAKE_INSTALL_PREFIX}
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib
includedir=\${prefix}/include

Name: github_user_fetcher
Description: My github_user_fetcher library
Version: ${PROJECT_VERSION}
Libs: -L\${libdir} -lgithub_user_fetcher
Cflags: -I\${includedir}
"
)

install(TARGETS github_user_fetcher github_user_fetcher_gui github_user_fetcher_shared github_user_fetcher_static
        RUNTIME DESTINATION bin
        LIBRARY DESTINATION lib
        ARCHIVE DESTINATION lib)

install(FILES src/user.hpp DESTINATION include/github_user_fetcher)
install(FILES src/style.css DESTINATION share/github_user_fetcher)

install(FILES ${CMAKE_CURRENT_BINARY_DIR}/github_user_fetcher.pc
        DESTINATION lib/pkgconfig)

option(BUILD_TESTS "Build the tests" ON)
if(BUILD_TESTS)
    enable_testing()
    
    find_package(Catch2 QUIET)
    if(NOT Catch2_FOUND)
        pkg_check_modules(CATCH2 REQUIRED IMPORTED_TARGET catch2-with-main)
        add_library(Catch2::Catch2WithMain ALIAS PkgConfig::CATCH2)
    endif()
    
    find_package(CURL REQUIRED)
    
    set(TEST_SUITES
        unit
        api
        cli
    )
    
    set(TEST_SOURCES_unit
        tests/unit/test_user.cpp
        tests/unit/test_main.cpp
    )
    
    set(TEST_SOURCES_api
        tests/api/test_server.cpp
    )
    
    set(TEST_SOURCES_cli
        tests/cli/test_cli.cpp
    )
    
    foreach(SUITE ${TEST_SUITES})
        add_executable(test_${SUITE} ${TEST_SOURCES_${SUITE}})
        target_link_libraries(test_${SUITE} PRIVATE
            github_user_fetcher_static
            Catch2::Catch2WithMain
            CURL::libcurl
            ${DEPENDENCIES}
        )
        add_test(NAME ${SUITE}_tests COMMAND test_${SUITE})
        set_tests_properties(${SUITE}_tests PROPERTIES LABELS ${SUITE})
        
        if(${SUITE} STREQUAL "cli")
            add_dependencies(test_${SUITE} github_user_fetcher)
        endif()
    endforeach()
endif()

option(BUILD_BENCHMARKS "Build the benchmarks" OFF)
if(BUILD_BENCHMARKS)
    find_package(benchmark QUIET)
    if(NOT benchmark_FOUND)
        pkg_check_modules(BENCHMARK REQUIRED IMPORTED_TARGET benchmark)
        add_library(benchmark::benchmark ALIAS PkgConfig::BENCHMARK)
    endif()
    
    add_executable(benchmarks benchmarks/bench_user.cpp)
    target_link_libraries(benchmarks PRIVATE
        github_user_fetcher_static
        benchmark::benchmark
        ${DEPENDENCIES}
    )
    
    add_test(NAME benchmarks COMMAND benchmarks)
    set_tests_properties(benchmarks PROPERTIES LABELS benchmark)
endif()
