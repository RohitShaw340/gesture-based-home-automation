package routes

import (
	"net/http"
	"os/exec"
)

func HandleCaptureCalibrationImages(w http.ResponseWriter, r *http.Request){
	runner := exec.Command("python","capture_calibration_images.py")
	output,err := runner.CombinedOutput()
	if(err!=nil){
		w.Write([]byte("unable to capture images for calibration"))
	}
	w.Write(output)
}

func HandleStereoCalibration(w http.ResponseWriter, r *http.Request){
	runner := exec.Command("python","stereo_calibration.py")
	output,err := runner.CombinedOutput()
	if(err!=nil){
		w.Write([]byte("unable to calibrate"))
	}
	w.Write(output)
}