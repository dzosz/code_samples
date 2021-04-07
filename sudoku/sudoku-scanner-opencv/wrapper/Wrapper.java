package sudokusolver;

import com.sun.jna.Library;
import com.sun.jna.Native;

public class Wrapper
{
    private interface CLibrary extends Library {
	    CLibrary INSTANCE = (CLibrary) Native.load("solver", CLibrary.class);
	    boolean solve(int[] grid, int elements);
    }

   public static boolean solve(int[] grid, int elements)
   {
	   return CLibrary.INSTANCE.solve(grid, elements);
   }
}
