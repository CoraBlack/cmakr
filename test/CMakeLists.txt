cmake_minimum_required(VERSION 3.20)

project(test_lib)

add_library(${PROJECT_NAME} SHARED "./func.c")