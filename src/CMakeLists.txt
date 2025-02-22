add_executable(zenith)

target_sources(zenith
    PUBLIC
    "main.cpp"
    "VulkanEngine.cpp"
    "VulkanInitializers.cpp"
)

if(MSVC)
    target_compile_options(zenith PRIVATE /W4 /sdl /permissive-)
else()
    target_compile_options(zenith PRIVATE -Wall -Wextra -Wpedantic)
endif()

target_link_libraries(zenith PRIVATE 
    spdlog::spdlog 
    volk::volk 
    vk-bootstrap::vk-bootstrap 
    GPUOpen::VulkanMemoryAllocator 
    SDL3::SDL3 
    fastgltf::fastgltf 
    glm::glm
)
