add_executable(
    zenith
    "main.cpp"
    "VulkanEngine.cpp"
)
if(MSVC)
    target_compile_options(zenith PRIVATE /W4 /WX /sdl /permissive-)
else()
    target_compile_options(zenith PRIVATE -Wall -Wextra -Wpedantic -Werror)
endif()

target_link_libraries(zenith PRIVATE spdlog::spdlog volk::volk vk-bootstrap::vk-bootstrap GPUOpen::VulkanMemoryAllocator SDL3::SDL3 fastgltf::fastgltf glm::glm)