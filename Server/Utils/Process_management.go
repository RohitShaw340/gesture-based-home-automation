package Utils

import (
	"errors"
	"fmt"
	"os"
	"sync"
)

// Kills a process using pid
func KillProcessByid(pid int) error {
	process, err := os.FindProcess(pid)
	if err != nil {
		return err
	}
	process.Kill()
	return nil
}

// Check if Service already running and kills it
func CheckAndKillService(name string, process_status *map[string]int, m *sync.Mutex) error {

	pid, ok := (*process_status)[name]
	if ok {
		err := KillProcessByid(pid)
		if err != nil {
			fmt.Println(err)
			return errors.New("service already running !!! Unable to restrt it")
		} else {
			m.Lock()
			delete(*process_status, name)
			m.Unlock()
		}
		return nil
	}
	return nil
}
