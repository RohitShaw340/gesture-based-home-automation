package main

import (
	"net/http"
	"server/routes"

	"github.com/gorilla/mux"
)

func main() {
	router := mux.NewRouter()
	router.HandleFunc("/", routes.GetHome)
	router.HandleFunc("/capture",routes.HandleCaptureCalibrationImages)
	router.HandleFunc("/calibrate",routes.HandleStereoCalibration)
	router.HandleFunc("/test1", routes.Test1)
	router.HandleFunc("/test2", routes.Test2)
	router.HandleFunc("/cancle", routes.Cancel)
	router.HandleFunc("/status", routes.Status)

	err:=http.ListenAndServe(":4000",router)

	if(err!=nil){
		panic("Unable to start server on port 4000")
	}
}