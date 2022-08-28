cmake_minimum_required(VERSION 3.10)

set (CMAKE_SYSTEM_VERSION 10.0 (latest installed version))
set (CMAKE_WINDOWS_KITS_10_DIR "C:/Program Files (x86)/Windows Kits/10")

# set the project name
project(libwinring C CXX)

# specify the C standard
set(CMAKE_C_STANDARD 17)
# set(CMAKE_C_STANDARD_REQUIRED True)
# specify the C++ standard
set(CMAKE_CXX_STANDARD 20)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS}")
set(CMAKE_EXE_LINKER_FLAGS_RELEASE "-Wl,-O3")

# add the executable
add_library(winring src/libwinring.cpp)

include_directories("include")
link_libraries("ntdll")
