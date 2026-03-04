package main

import (
    "encoding/json"
    "fmt"
    "log"
    "net/http"
)

type Response struct {
    Message string `json:"message"`
}

func helloHandler(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Content-Type", "application/json")
    response := Response{Message: "Hello from Go HTTP server!"}
    json.NewEncoder(w).Encode(response)
}

func main() {
    http.HandleFunc("/", helloHandler)
    fmt.Println("Go server running on :8080")
    log.Fatal(http.ListenAndServe(":8080", nil))
}
