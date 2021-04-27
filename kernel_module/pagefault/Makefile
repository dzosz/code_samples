all: page_fault do

.phony: do

do:
	time -v ./page_fault /usr/bin/git

page_fault: page_fault.c
	gcc page_fault.c -o page_fault

