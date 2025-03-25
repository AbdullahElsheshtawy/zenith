find_package(Vulkan REQUIRED)
include(FetchContent)


set(SDL_TESTS OFF)
FetchContent_Declare(
	SDL
	GIT_REPOSITORY https://github.com/libsdl-org/SDL
	GIT_TAG release-3.2.8
)
FetchContent_MakeAvailable(SDL)

FetchContent_Declare(
	vk_bootstrap
	GIT_REPOSITORY https://github.com/charles-lunarg/vk-bootstrap
	GIT_TAG v1.4.310
)
FetchContent_MakeAvailable(vk_bootstrap)

FetchContent_Declare(
	volk
	GIT_REPOSITORY https://github.com/zeux/volk
	GIT_TAG 1.4.304
)
FetchContent_MakeAvailable(volk)

FetchContent_Declare(
	vma
	GIT_REPOSITORY https://github.com/GPUOpen-LibrariesAndSDKs/VulkanMemoryAllocator
	GIT_TAG v3.2.1
)
FetchContent_MakeAvailable(vma)

FetchContent_Declare(
	spdlog
	GIT_REPOSITORY https://github.com/gabime/spdlog
	GIT_TAG v1.15.1
)
FetchContent_MakeAvailable(spdlog)


FetchContent_Declare(
	glm
	GIT_REPOSITORY https://github.com/g-truc/glm/
	GIT_TAG 1.0.1
)
FetchContent_MakeAvailable(glm)

FetchContent_Declare(
	imgui
	GIT_REPOSITORY https://github.com/ocornut/imgui
	GIT_TAG v1.91.9b
)
FetchContent_GetProperties(imgui)
if(NOT imgui_POPULATED)
	message("Fetching imgui")
	FetchContent_Populate(imgui)
	add_library(imgui
		${imgui_SOURCE_DIR}/imgui.cpp
		${imgui_SOURCE_DIR}/imgui_demo.cpp
		${imgui_SOURCE_DIR}/imgui_draw.cpp
		${imgui_SOURCE_DIR}/imgui_widgets.cpp
		${imgui_SOURCE_DIR}/imgui_tables.cpp
		${imgui_SOURCE_DIR}/backends/imgui_impl_sdl3.cpp
		${imgui_SOURCE_DIR}/backends/imgui_impl_vulkan.cpp
	)
	target_compile_definitions(imgui PUBLIC IMGUI_IMPL_VULKAN_USE_VOLK=1 IMGUI_IMPL_VULKAN_NO_PROTOTYPES=1)
	target_include_directories(imgui PUBLIC ${imgui_SOURCE_DIR} ${imgui_SOURCE_DIR}/backends)
	target_link_libraries(imgui PUBLIC SDL3::SDL3 volk::volk)
endif()

add_library(Vma)
target_sources(Vma PRIVATE "vma/vma.cpp" "vma/include/vma.hpp")
target_include_directories(Vma PUBLIC "vma/include/")
target_link_libraries(Vma PUBLIC volk::volk GPUOpen::VulkanMemoryAllocator)
target_compile_options(Vma PRIVATE -WnoEverything)
