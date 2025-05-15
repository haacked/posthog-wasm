package main

import "os"

func main() {
	config := PosthogClientConfig{
		ApiKey: os.Getenv("POSTHOG_API_KEY"),
		Host:   os.Getenv("POSTHOG_HOST"),
	}
	client := NewClient(config)
	client.Capture(PosthogEvent{
		Event: "web assembly event",
		Properties: map[string]any{
			"$lib":            "posthog-wasm",
			"$lib_version":    "0.1.0",
			"$geoip_disabled": true,
			"distinct_id":     "123412323",
		},
	})
	client.Flush()
	print("Event captured successfully")
}
