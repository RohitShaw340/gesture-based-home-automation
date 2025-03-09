package main

import (
	"net/http"
	"server/routes"

	"github.com/gorilla/mux"
)

func main() {
	router := mux.NewRouter()

	// CORS middleware
	router.Use(mux.CORSMethodMiddleware(router))
	router.Use(func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Access-Control-Allow-Origin", "*")
			w.Header().Set("Access-Control-Allow-Methods", "GET, POST, OPTIONS, PUT, DELETE")
			w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization")
			if r.Method == "OPTIONS" {
				w.WriteHeader(http.StatusOK)
				return
			}
			next.ServeHTTP(w, r)
		})
	})

	router.HandleFunc("/", routes.GetHome)
	router.HandleFunc("/capture", routes.HandleCaptureCalibrationImages)
	router.HandleFunc("/calibrate", routes.HandleStereoCalibration)
	router.HandleFunc("/test1", routes.Test1)
	router.HandleFunc("/test2", routes.Test2)
	router.HandleFunc("/cancle", routes.Cancel)
	router.HandleFunc("/status", routes.Status)
	router.HandleFunc("/rotate", routes.RotateCamera).Methods("OPTIONS", "POST")

	err := http.ListenAndServe(":4000", router)

	if err != nil {
		panic("Unable to start server on port 4000")
	}
}
