add_executable(zenith)

option(ZENITH_DEV "Development Mode" OFF)

target_sources(zenith PRIVATE
    "main.cpp"
)


add_subdirectory(engine)
target_link_libraries(zenith PRIVATE engine SDL3::SDL3 spdlog::spdlog)

if(ZENITH_DEV)
target_compile_options(zenith PRIVATE -Wall -Wextra -Werror)
else()
target_compile_options(zenith PRIVATE -Wall -Wextra)
endif()