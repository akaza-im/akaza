cmake_minimum_required(VERSION 3.17)
project(libakaza)

include(GNUInstallDirs)

set(CMAKE_CXX_STANDARD 17)
SET(CMAKE_C_FLAGS "-Wall -O2 -g ${CC_WARNING_FLAGS} ${CMAKE_C_FLAGS}")

# =============================================================================================
#
# library
#
# =============================================================================================

add_library(akaza SHARED include/tinylisp.h src/tinylisp.cc)

install(TARGETS akaza
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
        PUBLIC_HEADER DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})
file(GLOB HEADERS include/*.h)
install(FILES ${HEADERS} DESTINATION include/akaza/)

# =============================================================================================
#
# bin
#
# =============================================================================================
add_executable(akaza-make-system-lm bin/akaza-make-system-lm.cc)
target_link_libraries(akaza-make-system-lm marisa)
install(TARGETS akaza-make-system-lm DESTINATION bin/)

add_executable(akaza-make-binary-dict bin/akaza-make-binary-dict.cc)
target_link_libraries(akaza-make-binary-dict marisa)
install(TARGETS akaza-make-binary-dict DESTINATION bin/)

# =============================================================================================
#
# pkg-config
#
# =============================================================================================

configure_file(akaza.pc.in akaza.pc @ONLY)
install(FILES ${CMAKE_BINARY_DIR}/akaza.pc DESTINATION ${CMAKE_INSTALL_DATAROOTDIR}/pkgconfig)

# =============================================================================================
#
# test
#
# =============================================================================================

add_executable(00_systemlm.t t/00_systemlm.cc)
set_target_properties(00_systemlm.t PROPERTIES RUNTIME_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/t/")
target_link_libraries(00_systemlm.t marisa)
SET(TEST_EXES 00_systemlm.t)

add_executable(01_binary_dict.t t/01_binary_dict.cc)
set_target_properties(01_binary_dict.t PROPERTIES RUNTIME_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/t/")
target_link_libraries(01_binary_dict.t marisa)
SET(TEST_EXES ${TEST_EXES} 01_binary_dict.t)

add_executable(02_skk_dict.t t/02_skk_dict.cc)
set_target_properties(02_skk_dict.t PROPERTIES RUNTIME_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/t/")
target_link_libraries(02_skk_dict.t marisa)
SET(TEST_EXES ${TEST_EXES} 02_skk_dict.t)

add_executable(03_tinylisp.t t/03_tinylisp.cc)
set_target_properties(03_tinylisp.t PROPERTIES RUNTIME_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/t/")
target_link_libraries(03_tinylisp.t marisa akaza)
SET(TEST_EXES ${TEST_EXES} 03_tinylisp.t)


ADD_CUSTOM_TARGET(test env BINARY_DIR=${CMAKE_CURRENT_BINARY_DIR} prove --exec '' -v t/*.t
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR} DEPENDS ${TEST_EXES})
