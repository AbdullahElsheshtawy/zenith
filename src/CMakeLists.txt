add_executable(zenith)

target_sources(zenith PRIVATE
	"main.cpp"
)

if(MSVC)
	target_compile_options(zenith PRIVATE -W4)
else()
	target_compile_options(zenith PRIVATE -Wall -Wextra)
endif()

add_subdirectory(engine)
target_link_libraries(zenith PRIVATE engine SDL3::SDL3 spdlog::spdlog)