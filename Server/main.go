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

	err:=http.ListenAndServe(":4000",router)

	if(err!=nil){
		panic("Unable to start server on port 4000")
	}
}