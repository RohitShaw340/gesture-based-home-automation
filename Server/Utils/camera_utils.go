package Utils

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"server/constants"
)

type cameraConfig struct {
	Pin             int `json:"pin"`
	CurrentPosition int `json:"current_position"`
}

type ServoConfig struct {
	Cam1 cameraConfig `json:"cam1"`
	Cam2 cameraConfig `json:"cam2"`
}

func CallCameraRotationScript(currentPosition int, gpioPinName string) error {
	absoluteAngle := fmt.Sprintf("%v", currentPosition)
	runner := exec.Command("./rotate_camera", "-a", absoluteAngle, "-p", gpioPinName)
	err := runner.Start()
	if err != nil {
		return fmt.Errorf("Failed to rotate camera to angle %v : %v", absoluteAngle, err)
	}

	fmt.Println("Waiting for camera rotation to complete...")

	err = runner.Wait()
	if err != nil {
		return fmt.Errorf("Failed to rotate camera to angle %v : %v", absoluteAngle, err)
	}
	output, err := runner.CombinedOutput()
	if err != nil {
		return fmt.Errorf("Failed to rotate camera to angle %v : %v", absoluteAngle, err)
	}

	fmt.Printf("Script output: %s\n", output)

	return nil
}

func TakePicture() error {
	fmt.Println("Taking picture...")

	// delayStr := fmt.Sprintf("%d", delayMilli)

	runner := exec.Command(
		"python",
		"../picam/take_picture.py",
		"-o",
		constants.ROTATION_IMAGE_DIR,
		"-f",
		constants.ROTATION_IMAGE_FILE_NAME,
		// "-d",
		// delayStr,
	)
	err := runner.Start()
	if err != nil {
		return fmt.Errorf("Failed to take picture: %v", err)
	}

	err = runner.Wait()
	if err != nil {
		return fmt.Errorf("Failed to take picture: %v", err)
	}

	fmt.Println("Picture taken successfully")

	return nil
}

// Load current servo position
func GetServoConfig() (ServoConfig, error) {
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
func SetPosition(position ServoConfig) error {
	file, err := os.Create("servo_config.json")
	if err != nil {
		return fmt.Errorf("Failed to save position: %v", err)
	}
	defer file.Close()

	err = json.NewEncoder(file).Encode(position)
	if err != nil {
		return fmt.Errorf("Failed to save position: %v", err)
	}

	return nil
}

func GetPositionAndPinFromConfig(
	camera_id, stepSize int,
	direction string,
	servoConfig ServoConfig,
) (int, string, error) {
	var currentPosition int

	// Select the correct GPIO pin
	var gpioPinName string

	switch camera_id {
	case 1:
		updatedPosition, err := UpdateCameraPosition(
			servoConfig.Cam1.CurrentPosition,
			stepSize,
			direction,
		)
		if err != nil {
			return 0, "", fmt.Errorf("Unable to update camera position: %w", err)
		}
		servoConfig.Cam1.CurrentPosition = updatedPosition
		gpioPinName = fmt.Sprintf("%v", servoConfig.Cam1.Pin)
		currentPosition = servoConfig.Cam1.CurrentPosition

	case 2:
		updatedPosition, err := UpdateCameraPosition(
			servoConfig.Cam2.CurrentPosition,
			stepSize,
			direction,
		)
		if err != nil {
			return 0, "", fmt.Errorf("Unable to update camera position: %w", err)
		}
		servoConfig.Cam2.CurrentPosition = updatedPosition
		gpioPinName = fmt.Sprintf("%v", servoConfig.Cam2.Pin)
		currentPosition = servoConfig.Cam2.CurrentPosition
	default:
		return 0, "", fmt.Errorf("Invalid camera ID: %v", camera_id)
	}

	return currentPosition, gpioPinName, nil
}

func UpdateCameraPosition(currentPosition int, stepSize int, direction string) (int, error) {
	var updatedPosition int

	// Update position based on direction
	if direction == "clock" {
		updatedPosition += stepSize
	} else if direction == "anticlock" {
		updatedPosition -= stepSize
	} else {
		return 0, fmt.Errorf("Invalid direction: %v", direction)
	}

	// Check if the new position is within the valid range
	if updatedPosition < -80 || updatedPosition > 80 {
		return 0, fmt.Errorf("Invalid position: %v", updatedPosition)
	}

	return updatedPosition, nil
}
