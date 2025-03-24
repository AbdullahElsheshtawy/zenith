find_program(SLANGC slangc HINTS /usr/bin /usr/local/bin $ENV{VULKAN_SDK}/Bin/ $ENV{VULKAN_SDK}/Bin32/)

if(NOT SLANGC) 
	message(FATAL_ERROR "Could not find slangc compiler")
endif()

file(GLOB_RECURSE shader_files
	"${CMAKE_CURRENT_SOURCE_DIR}/*.slang"
)

add_custom_target(Shaders ALL)

foreach(shader ${shader_files})
	get_filename_component(shader_name ${shader} NAME_WE)

	add_custom_command(
		TARGET Shaders
		COMMAND "${SLANGC} -o ${CMAKE_CURRENT_SOURCE_DIR}/${shader_name}.spv ${shader}"
		DEPENDS ${shader}
		COMMENT "Compiling shader: ${shader_name}"
	)

endforeach()

