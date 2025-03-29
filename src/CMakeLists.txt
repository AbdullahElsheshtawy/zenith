add_executable(zenith)

option(ZENITH_DEV "Development Mode(Warnings as Errors)" OFF)

target_sources(zenith PRIVATE
    "main.cpp"
)


add_subdirectory(engine)
target_link_libraries(zenith PRIVATE engine SDL3::SDL3 spdlog::spdlog)

if(ZENITH_DEV)
    if(MSVC)
        target_compile_options(zenith PRIVATE /W4 /WX)
    else()
        target_compile_options(zenith PRIVATE -Wall -Wextra -Werror)
    endif()
else()
    if(MSVC)
        target_compile_options(zenith PRIVATE /W4)
    else()
        target_compile_options(zenith PRIVATE -Wall -Wextra)
    endif()
endif()