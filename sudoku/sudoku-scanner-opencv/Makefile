CLASSPATH = -cp $$PWD/externals/:$$PWD/externals/*:$$PWD/sudokusolver/:/usr/share/java/opencv.jar:.

SOLVABLE = 7 2 3 0 0 0 1 5 9 6 0 0 3 0 2 0 0 8 8 0 0 0 1 0 0 0 2 0 7 0 6 5 4 0 2 0 0 0 4 2 0 7 3 0 0 0 5 0 9 3 1 0 4 0 5 0 0 0 7 0 0 0 3 4 0 0 1 0 3 0 0 6 9 3 2 0 0 0 7 1 4

.PHONY = all run clean test externals

all: sudokusolver/Wrapper.class sudokusolver/Solve.class sudokusolver/Detection.class

externals/jna-5.5.0.jar:
	@mkdir -p externals
	wget http://repo1.maven.org/maven2/net/java/dev/jna/jna/5.5.0/jna-5.5.0.jar -O externals/jna-5.5.0.jar

externals/libsolver.so: ../sudoku_solver/libsolver.so
	@mkdir -p externals
	@cp ../sudoku_solver/libsolver.so externals/
	
externals: externals/jna-5.5.0.jar externals/libsolver.so
	
sudokusolver/%.class: wrapper/%.java externals
	javac -d $$PWD $(CLASSPATH) $<

sudokusolver/Detection.class: detection/Detection.java externals
	javac -d $$PWD $(CLASSPATH) $<

run:  sudokusolver/Detection.class
	java $(CLASSPATH) sudokusolver.Detection $(ARGS)

clean:
	rm -rf ./sudokusolver

test:
	java $(CLASSPATH) sudokusolver.Solve $(SOLVABLE)
