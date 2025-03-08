package routes

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"

	"github.com/stianeikeland/go-rpio/v4"
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

func getCurrentPosition() (ServoConfig, error) {
	var position ServoConfig

	file, err := os.Open("/servo_config.json")
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

func setPositon(position ServoConfig) error {
	file, err := os.Create("/servo_config.json")
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

	ServoConfig, err := getCurrentPosition()
	if err != nil {
		http.Error(w, "Unable to read current position", http.StatusInternalServerError)
		return
	}

	if req.Direction == "clock" {
		ServoConfig.CurrentPosition += req.StepSize
	} else if req.Direction == "anticlock" {
		ServoConfig.CurrentPosition -= req.StepSize
	} else {
		http.Error(w, "Invalid direction", http.StatusBadRequest)
		return
	}

	var pinNumber int

	switch req.Camera_id {
	case 1:
		pinNumber = ServoConfig.Cam1Pin
	case 2:
		pinNumber = ServoConfig.Cam2Pin
	default:
		http.Error(w, "Invalid camera id", http.StatusBadRequest)
		return
	}

	err = rpio.Open()
	if err != nil {
		http.Error(w, "Unable to open GPIO", http.StatusInternalServerError)
		return
	}
	defer rpio.Close()

	pin := rpio.Pin(pinNumber)
	pin.Mode(rpio.Pwm)
	pin.Freq(64000)
	pin.DutyCycle(uint32(ServoConfig.CurrentPosition), 100)
	// pin.Write(uint8(currentPosition))

	fmt.Fprintf(w, "Servo rotated to position: %d", ServoConfig.CurrentPosition)

	err = setPositon(ServoConfig)
	if err != nil {
		http.Error(w, "Unable to set position", http.StatusInternalServerError)
		return
	}
}
