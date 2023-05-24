cmake_minimum_required(VERSION 3.5)

project(sto_locking LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(testing testing.cpp lockable_item.cpp TransactionManager.cpp)
