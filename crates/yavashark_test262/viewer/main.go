package main

import (
	"log"
	"viewer/cache"
	"viewer/router"
)

func main() {
	if err := cache.InitWithCurrent(); err != nil {
		log.Fatalf("Failed to initialize cache: %v", err)
	}
	router.Start()
}
