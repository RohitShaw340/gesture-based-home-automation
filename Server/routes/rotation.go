package routes

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"os/exec"
)

const (
	ROTATION_IMAGE_FILE_NAME = "rotation.jpeg"
	ROTATION_IMAGE_DIR       = "../Server/rotation_images"
)

type RotationRequest struct {
	Camera_id int    `json:"camera_id"`
	StepSize  int    `json:"step_size"`
	Direction string `json:"direction"`
}

type ServoConfig struct {
	Cam1Pin         int `json:"cam1_pin"`
	Cam2Pin         int `json:"cam2_pin"`
	CurrentPosition int `json:"current_position"`
}

// Load current servo position
func getCurrentPosition() (ServoConfig, error) {
	var position ServoConfig

	file, err := os.Open("servo_config.json")
	if err != nil {
		return ServoConfig{}, err
	}
	defer file.Close()

	err = json.NewDecoder(file).Decode(&position)
	if err != nil {
		return ServoConfig{}, err
	}

	return position, nil
}

// Save new servo position
func setPosition(position ServoConfig) error {
	file, err := os.Create("servo_config.json")
	if err != nil {
		return err
	}
	defer file.Close()

	err = json.NewEncoder(file).Encode(position)
	if err != nil {
		return err
	}

	return nil
}

func RotateCamera(w http.ResponseWriter, r *http.Request) {
	var req RotationRequest
	err := json.NewDecoder(r.Body).Decode(&req)
	if err != nil {
		http.Error(w, "Invalid request payload", http.StatusBadRequest)
		return
	}

	// Get current position
	servoConfig, err := getCurrentPosition()
	if err != nil {
		http.Error(w, "Unable to read current position", http.StatusInternalServerError)
		return
	}

	// Update position based on direction
	if req.Direction == "clock" {
		servoConfig.CurrentPosition += req.StepSize
	} else if req.Direction == "anticlock" {
		servoConfig.CurrentPosition -= req.StepSize
	} else {
		http.Error(w, "Invalid direction", http.StatusBadRequest)
		return
	}

	// Ensure position is within servo limits (-80 to 80 degrees)
	if servoConfig.CurrentPosition < -80 || servoConfig.CurrentPosition > 80 {
		fmt.Printf("Servo limit reached: %v", servoConfig.CurrentPosition)
		http.Error(w, "Servo limit reached", http.StatusBadRequest)
		return
		// servoConfig.CurrentPosition = 80
	}

	// Select the correct GPIO pin
	var gpioPinName string
	switch req.Camera_id {
	case 1:
		gpioPinName = fmt.Sprintf("%v", servoConfig.Cam1Pin)
	case 2:
		gpioPinName = fmt.Sprintf("%v", servoConfig.Cam2Pin)
	default:
		fmt.Printf("Invalid camera ID: %v", req.Camera_id)
		http.Error(w, "Invalid camera ID", http.StatusBadRequest)
		return
	}

	absoluteAngle := fmt.Sprintf("%v", servoConfig.CurrentPosition)

	runner := exec.Command("./rotate_camera", "-a", absoluteAngle, "-p", gpioPinName)
	err = runner.Start()
	if err != nil {
		fmt.Printf("Failed to rotate camera: %v", err)
		http.Error(w, "Failed to rotate camera", http.StatusInternalServerError)
		return
	}

	err = runner.Wait()
	if err != nil {
		fmt.Printf("Failed to rotate camera: %v", err)
		http.Error(w, "Failed to rotate camera", http.StatusInternalServerError)
		return
	}

	runner = exec.Command("python", "../picam/take_picture.py", "-o", ROTATION_IMAGE_DIR, "-f", ROTATION_IMAGE_FILE_NAME)
	err = runner.Start()
	if err != nil {
		fmt.Printf("Failed to take picture: %v", err)
		http.Error(w, "Failed to take picture", http.StatusInternalServerError)
		return
	}

	err = runner.Wait()
	if err != nil {
		fmt.Printf("Failed to take picture: %v", err)
		http.Error(w, "Failed to take picture", http.StatusInternalServerError)
		return
	}

	switch req.Camera_id {
	case 1:
		http.ServeFile(w, r, fmt.Sprintf("%s/cam1/%s", ROTATION_IMAGE_DIR, ROTATION_IMAGE_FILE_NAME))
	case 2:
		http.ServeFile(w, r, fmt.Sprintf("%s/cam2/%s", ROTATION_IMAGE_DIR, ROTATION_IMAGE_FILE_NAME))
	default:
		http.Error(w, "Invalid camera ID", http.StatusBadRequest)
		return
	}

	// Save new position
	err = setPosition(servoConfig)
	if err != nil {
		fmt.Printf("Failed to save position: %v", err)
		http.Error(w, "Unable to save position", http.StatusInternalServerError)
		return
	}
}
