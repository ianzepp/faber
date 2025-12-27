cmake_minimum_required(VERSION 3.20)
project(http-demo CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Fetch Crow (header-only HTTP framework)
include(FetchContent)
FetchContent_Declare(
    crow
    GIT_REPOSITORY https://github.com/CrowCpp/Crow.git
    GIT_TAG v1.0+5
)
FetchContent_MakeAvailable(crow)

# Main executable
add_executable(http-demo src/main.cpp)
target_link_libraries(http-demo PRIVATE Crow::Crow)

# Enable warnings
target_compile_options(http-demo PRIVATE
    $<$<CXX_COMPILER_ID:GNU,Clang,AppleClang>:-Wall -Wextra>
)
