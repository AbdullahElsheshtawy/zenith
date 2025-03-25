add_library(engine)

target_sources(engine PRIVATE
    "types.hpp"	
    "engine.hpp"
    "engine.cpp"
    "util.hpp"
    "util.cpp"
    "deletion_queue.hpp"
    "deletion_queue.cpp"
    "descriptors.hpp"
    "descriptors.cpp"
)


add_dependencies(engine Shaders)

# Stop volk.h and vulkan.h symbol conflicts.
# https://github.com/zeux/volk/tree/master?tab=readme-ov-file#basic-usage
target_compile_definitions(engine PUBLIC VK_NO_PROTOTYPES=1)

find_package(Vulkan REQUIRED)
target_link_libraries(engine PUBLIC Vulkan::Headers SDL3::SDL3 vk-bootstrap::vk-bootstrap volk::volk Vma glm::glm spdlog::spdlog imgui)

if(ZENITH_DEV)
target_compile_options(zenith PRIVATE -Wall -Wextra -Werror)
else()
target_compile_options(zenith PRIVATE -Wall -Wextra)
endif()