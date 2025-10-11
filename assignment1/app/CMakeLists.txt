# Build the app, using the HAL

include_directories(include)
file(GLOB MY_SOURCES "src/*.c")
add_executable(assignment1 ${MY_SOURCES})

# Make use of the HAL library
target_link_libraries(assignment1 LINK_PRIVATE hal)

# Copy executable to remote target via SCP after build
add_custom_command(TARGET assignment1 POST_BUILD
  COMMAND scp "$<TARGET_FILE:assignment1>" Matthew@10.0.2.106:/home/Matthew/bin/assignment1
  COMMENT "Copying ARM executable to remote host via SCP")