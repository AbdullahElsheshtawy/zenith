add_library(engine)

target_sources(engine PRIVATE
	"types.hpp"	
	"engine.hpp"
	"engine.cpp"
	"util.hpp"
	"util.cpp"
)

if(MSVC)
	target_compile_options(engine PRIVATE /W4)
else()
	target_compile_options(engine PRIVATE -Wall -Wextra)
endif()

add_dependencies(engine Shaders)

# Stop volk.h and vulkan.h symbol conflicts.
# https://github.com/zeux/volk/tree/master?tab=readme-ov-file#basic-usage
target_compile_definitions(engine PUBLIC VK_NO_PROTOTYPES=1)

find_package(Vulkan REQUIRED)
target_include_directories(engine PRIVATE ${VULKAN_INCLUDE_DIRS})
target_link_libraries(engine PUBLIC Vulkan::Headers SDL3::SDL3 vk-bootstrap::vk-bootstrap volk::volk GPUOpen::VulkanMemoryAllocator glm::glm spdlog::spdlog)