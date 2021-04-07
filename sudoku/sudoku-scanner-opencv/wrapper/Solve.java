package sudokusolver;

import static java.lang.System.out;
import java.util.Arrays; 
import sudokusolver.Wrapper;

public class Solve
{
   public static void main(final String[] argv)
   {
	   int[] grid = new int [argv.length];
	   for(int i=0; i<argv.length; i++) {
		   grid[i] = Integer.parseInt(argv[i]);
	   }
	   boolean solved = Wrapper.solve(grid, argv.length);
	   out.println((solved ? "SOLVED" : "UNSOLVABLE"));
	   out.println(Arrays.toString(grid));
   }
}
