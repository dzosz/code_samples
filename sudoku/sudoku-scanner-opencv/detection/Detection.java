package sudokusolver;

import org.opencv.core.Core;
import org.opencv.imgcodecs.Imgcodecs;
import org.opencv.imgproc.Imgproc;
import org.opencv.core.Mat;
import org.opencv.core.Size;
import org.opencv.core.Scalar;
import org.opencv.core.MatOfPoint;
import java.util.ArrayList;
import java.util.List;


import static java.lang.System.out;

class Detection 
{
	public static void main(String[] args)
	{
		System.loadLibrary(Core.NATIVE_LIBRARY_NAME);
		Mat src = Imgcodecs.imread("input.jpg", Imgcodecs.CV_LOAD_IMAGE_GRAYSCALE);
		Mat blur = new Mat();
		Mat edges = new Mat();

			
		Mat threshold = new Mat();
		Imgproc.adaptiveThreshold(src, threshold, 255, Imgproc.ADAPTIVE_THRESH_GAUSSIAN_C, Imgproc.THRESH_BINARY, 11, 12);
		//Imgcodecs.imwrite("adaptive.jpg", thre);
		// blur
		Imgproc.GaussianBlur(threshold, blur, new Size(11, 11), 0.0);
		//Imgcodecs.imwrite("blur.jpg", blur);

		//Imgproc.medianBlur(src, blur, 5);
		//Imgcodecs.imwrite("medianblur.jpg", blur);

		// canny 
		Imgproc.Canny(blur, edges, 0, 255);

		// contours
		final List<MatOfPoint> contours = new ArrayList<>();
		final Mat hierarchy = new Mat();
		//Imgproc.findContours(edges, contours, hierarchy, Imgproc.RETR_TREE, Imgproc.CHAIN_APPROX_SIMPLE);
		Imgproc.findContours(edges, contours, hierarchy, Imgproc.RETR_EXTERNAL, Imgproc.CHAIN_APPROX_SIMPLE);
		System.out.println("contours" + contours);
		Scalar white = new Scalar(255, 255, 255);
		Imgproc.drawContours(edges, contours, -1, white);
		
		/// write out
		Imgcodecs.imwrite("output.jpg", edges);
		System.out.println("written to output.jpg");
	}

}
