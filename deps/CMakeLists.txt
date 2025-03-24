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
