package routes

import (
	"encoding/json"
	"net/http"
	"os/exec"
	"server/Utils"
	"strconv"
)

type runner_status struct {
	Pids          []int
	Process_Names []string
}

// var Process_Status runner_status
var Process_Status = make(map[string]int)

func GetHome(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/html")
	w.Write([]byte("<H1>Welcome to Gesture Ease</H1>"))
}

func Test1(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/html")

	err := Utils.CheckAndKillService("Test1", &Process_Status)
	if err != nil {
		w.Write([]byte("Unable to restart service"))
		return
	}

	// Restart Service
	runner := exec.Command("python", "hello.py")
	err = runner.Start()
	if err != nil {
		w.Write([]byte("unable to capture images for calibration"))
	}

	Process_Status["Test1"] = runner.Process.Pid
	w.Write([]byte("Test 1 Launched"))
}

func Test2(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/html")

	// Kill Servise if already running
	err := Utils.CheckAndKillService("Test2", &Process_Status)
	if err != nil {
		w.Write([]byte("Unable to restart service"))
		return
	}

	// Restart Service
	runner := exec.Command("python", "hello.py")
	err = runner.Start()
	if err != nil {
		w.Write([]byte("unable to start Test2"))
		return
	}

	Process_Status["Test2"] = runner.Process.Pid
	w.Write([]byte("Test 2 Launched"))
}

func Status(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	status, err := json.Marshal(Process_Status)
	if err != nil {
		w.Write([]byte("Unable to find status of services"))
		return
	}

	w.Write(status)
}

func Cancel(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/html")

	// err := Utils.KillProcessByName("test1")
	// process, err := os.FindProcess(int(test1.pid))

	for service_name, pid := range Process_Status {
		err := Utils.CheckAndKillService(service_name, &Process_Status)
		if err != nil {
			w.Write([]byte(service_name + " : " + strconv.FormatInt(int64(pid), 10) + " -> Unable to Stop \n"))
		} else {
			w.Write([]byte(service_name + " : " + strconv.FormatInt(int64(pid), 10) + " -> Stopped \n"))
		}
	}

}
