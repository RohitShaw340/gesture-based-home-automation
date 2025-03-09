package routes

import (
	"encoding/json"
	"fmt"
	"net/http"
	"server/Utils"
	"server/constants"

	"github.com/gorilla/mux"
)

type RotationRequest struct {
	Camera_id int    `json:"camera_id"`
	StepSize  int    `json:"step_size"`
	Direction string `json:"direction"`
}

type ResetRequest struct {
	Camera_id int `json:"camera_id"`
}

func RotateCamera(w http.ResponseWriter, r *http.Request) {
	var req RotationRequest
	err := json.NewDecoder(r.Body).Decode(&req)
	if err != nil {
		http.Error(w, "Invalid request payload", http.StatusBadRequest)
		return
	}

	// Get current position
	servoConfig, err := Utils.GetServoConfig()
	if err != nil {
		http.Error(w, "Unable to read current position", http.StatusInternalServerError)
		return
	}

	currentPosition, gpioPinName, err := Utils.GetPositionAndPinFromConfig(req.Camera_id, req.StepSize, req.Direction, servoConfig)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	err = Utils.CallCameraRotationScript(currentPosition, gpioPinName)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	// Save new position
	err = Utils.SetPosition(servoConfig)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	err = Utils.TakePicture()
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	switch req.Camera_id {
	case 1:
		http.ServeFile(w, r, fmt.Sprintf("%s/cam1/%s", constants.ROTATION_IMAGE_DIR, constants.ROTATION_IMAGE_FILE_NAME))
	case 2:
		http.ServeFile(w, r, fmt.Sprintf("%s/cam2/%s", constants.ROTATION_IMAGE_DIR, constants.ROTATION_IMAGE_FILE_NAME))
	default:
		http.Error(w, "Invalid camera ID", http.StatusBadRequest)
		return
	}
}

func ResetCameraPosition(w http.ResponseWriter, r *http.Request) {
	var req ResetRequest
	err := json.NewDecoder(r.Body).Decode(&req)
	if err != nil {
		http.Error(w, "Invalid request payload", http.StatusBadRequest)
		return
	}

	servoConfig, err := Utils.GetServoConfig()
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	var gpioPinName string
	switch req.Camera_id {
	case 1:
		gpioPinName = fmt.Sprintf("%v", servoConfig.Cam1.Pin)
		servoConfig.Cam1.CurrentPosition = 0

	case 2:
		gpioPinName = fmt.Sprintf("%v", servoConfig.Cam2.Pin)
		servoConfig.Cam2.CurrentPosition = 0

	default:
		fmt.Printf("Invalid camera ID: %v", req.Camera_id)
		http.Error(w, "Invalid camera ID", http.StatusBadRequest)
		return
	}

	err = Utils.CallCameraRotationScript(0, gpioPinName)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	// Save new position
	err = Utils.SetPosition(servoConfig)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	err = Utils.TakePicture()
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	switch req.Camera_id {
	case 1:
		http.ServeFile(w, r, fmt.Sprintf("%s/cam1/%s", constants.ROTATION_IMAGE_DIR, constants.ROTATION_IMAGE_FILE_NAME))
	case 2:
		http.ServeFile(w, r, fmt.Sprintf("%s/cam2/%s", constants.ROTATION_IMAGE_DIR, constants.ROTATION_IMAGE_FILE_NAME))
	default:
		http.Error(w, "Invalid camera ID", http.StatusBadRequest)
		return
	}

}

func GetCameraPosition(w http.ResponseWriter, r *http.Request) {
	servoConfig, err := Utils.GetServoConfig()
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	// Send response
	w.Header().Set("Content-Type", "application/json")
	err = json.NewEncoder(w).Encode(servoConfig)
	if err != nil {
		http.Error(w, "Unable to encode response", http.StatusInternalServerError)
		return
	}
}

func GetCameraPicture(w http.ResponseWriter, r *http.Request) {
	vars := mux.Vars(r)
	cameraID := vars["camera-id"]

	var imagePath string

	switch cameraID {
	case "1":
		imagePath = fmt.Sprintf("%s/cam1/%s", constants.ROTATION_IMAGE_DIR, constants.ROTATION_IMAGE_FILE_NAME)
	case "2":
		imagePath = fmt.Sprintf("%s/cam2/%s", constants.ROTATION_IMAGE_DIR, constants.ROTATION_IMAGE_FILE_NAME)
	default:
		http.Error(w, "Invalid camera ID", http.StatusBadRequest)
		return
	}

	http.ServeFile(w, r, imagePath)
}
