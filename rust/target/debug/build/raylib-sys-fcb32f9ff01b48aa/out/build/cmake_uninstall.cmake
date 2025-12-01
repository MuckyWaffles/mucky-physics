if(NOT EXISTS "/home/muckywaffles/Programming/mucky-physics/rust/target/debug/build/raylib-sys-fcb32f9ff01b48aa/out/build/install_manifest.txt")
  message(FATAL_ERROR "Cannot find install manifest: /home/muckywaffles/Programming/mucky-physics/rust/target/debug/build/raylib-sys-fcb32f9ff01b48aa/out/build/install_manifest.txt")
endif()

file(READ "/home/muckywaffles/Programming/mucky-physics/rust/target/debug/build/raylib-sys-fcb32f9ff01b48aa/out/build/install_manifest.txt" files)
string(REGEX REPLACE "\n" ";" files "${files}")
foreach(file ${files})
  message(STATUS "Uninstalling $ENV{DESTDIR}${file}")
  if(IS_SYMLINK "$ENV{DESTDIR}${file}" OR EXISTS "$ENV{DESTDIR}${file}")
    exec_program(
      "/nix/store/w9jm660dykns6hzrdhxmqfywnc9ail8g-cmake-4.1.2/bin/cmake" ARGS "-E remove \"$ENV{DESTDIR}${file}\""
      OUTPUT_VARIABLE rm_out
      RETURN_VALUE rm_retval
      )
    if(NOT "${rm_retval}" STREQUAL 0)
      message(FATAL_ERROR "Problem when removing $ENV{DESTDIR}${file}")
    endif()
  else(IS_SYMLINK "$ENV{DESTDIR}${file}" OR EXISTS "$ENV{DESTDIR}${file}")
    message(STATUS "File $ENV{DESTDIR}${file} does not exist.")
  endif()
endforeach()
