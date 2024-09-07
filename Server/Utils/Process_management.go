package Utils

import (
	"errors"
	"fmt"
	"os"
)

// func KillProcessByName(name string) error {
// 	// Get the process information by name
// 	output, _ := exec.Command("ps", "-o", "pid,comm", "-C", name).Output()
// 	lines := strings.Split(string(output), "\n")

// 	for i, line := range lines {
// 		if i == 0 {
// 			continue
// 		}
// 		fields := strings.Fields(line)
// 		if len(fields) < 2 {
// 			continue
// 		}
// 		fmt.Printf("Killing process: %s (PID: %s)\n", fields[1], fields[0])
// 		// Kill the process
// 		exec.Command("pkill", "-f", fields[0]).Run()
// 	}
// 	return nil
// }

func KillProcessByid(pid int) error {
	process, err := os.FindProcess(pid)
	if err != nil {
		return err
	}
	process.Kill()
	return nil
}

// Check if Service already running and kills it
func CheckAndKillService(name string, process_status *map[string]int) error {

	pid, ok := (*process_status)[name]
	if ok {
		err := KillProcessByid(pid)
		if err != nil {
			fmt.Println(err)
			return errors.New("service already running !!! Unable to restrt it")
		} else {
			delete(*process_status, name)
		}
		return nil
	}
	return nil
}
