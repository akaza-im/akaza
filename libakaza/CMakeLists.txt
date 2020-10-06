cmake_minimum_required(VERSION 3.17)
project(libakaza)

set(CMAKE_CXX_STANDARD 17)
SET(CMAKE_C_FLAGS "-Wall -O2 -g ${CC_WARNING_FLAGS} ${CMAKE_C_FLAGS}")

add_library(akaza SHARED include/tinylisp.h src/tinylisp.cc)

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
