find_program(SLANGC slangc HINTS /usr/bin /usr/local/bin $ENV{VULKAN_SDK}/Bin/ $ENV{VULKAN_SDK}/Bin32/)

if(NOT SLANGC) 
	message(FATAL_ERROR "Could not find slangc compiler")
endif()

set(shader_files
	"${CMAKE_CURRENT_SOURCE_DIR}/gradient_color.slang"
	"${CMAKE_CURRENT_SOURCE_DIR}/sky.slang"
)

add_custom_target(Shaders ALL)

foreach(shader ${shader_files})
	get_filename_component(shader_name ${shader} NAME_WE)

	add_custom_command(
		TARGET Shaders
		COMMAND ${SLANGC}
		ARGS -o ${CMAKE_CURRENT_BINARY_DIR}/${shader_name}.spv ${shader}
		DEPENDS ${shader}
		COMMENT "Compiling shader: ${shader_name}"
	)

endforeach()

