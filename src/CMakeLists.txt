add_executable(
	zenith
	"main.cpp"
)
if(MSVC)
	target_compile_options(zenith PRIVATE /W4 /WX /sdl /permissive-)
else()
	target_compile_options(zenith  "-Werror=vla, -Wall, -Wextra, -Wpedantic")
endif()
target_link_libraries(zenith PRIVATE spdlog::spdlog)