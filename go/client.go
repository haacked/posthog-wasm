package main

import (
	"context"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"os"

	extism "github.com/extism/go-sdk"
)

type PosthogClientConfig struct {
	ApiKey string `json:"api_key"`
	Host   string `json:"host"`
}

type PosthogClient struct {
	plugin    *extism.Plugin
	client_id uint32
}

func NewClient(ph_config PosthogClientConfig) *PosthogClient {
	manifest := extism.Manifest{
		AllowedHosts: []string{"*"},
		Wasm: []extism.Wasm{
			extism.WasmFile{
				Path: "./posthog_wasm.wasm",
			},
		},
	}
	plugin_config := extism.PluginConfig{
		EnableWasi: true,
	}
	ctx := context.Background()
	plugin, err := extism.NewPlugin(ctx, manifest, plugin_config, []extism.HostFunction{})
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	data, err := json.Marshal(ph_config)
	if err != nil {
		fmt.Println("Error marshaling capture input:", err)
		os.Exit(1)
	}
	exit, client_id_b, err := plugin.Call("create_client", data)
	if err != nil {
		fmt.Println(err)
		os.Exit(int(exit))
	}
	client_id := binary.BigEndian.Uint32(client_id_b)
	return &PosthogClient{
		plugin,
		client_id,
	}
}

type PosthogEvent struct {
	Event      string         `json:"event"`
	Properties map[string]any `json:"properties"`
}

type CaptureInput struct {
	ClientHandle uint32       `json:"handle"`
	Event        PosthogEvent `json:"event"`
}

func (c *PosthogClient) Capture(event PosthogEvent) {
	input := CaptureInput{
		ClientHandle: c.client_id,
		Event:        event,
	}
	data, err := json.Marshal(input)
	if err != nil {
		fmt.Println("Error marshaling capture input:", err)
		os.Exit(1)
	}
	exit, _, err := c.plugin.Call("capture", data)
	if err != nil {
		fmt.Println(err)
		os.Exit(int(exit))
	}
}

type FlushInput struct {
	ClientHandle uint32 `json:"handle"`
}

func (c *PosthogClient) Flush() {
	input := CaptureInput{
		ClientHandle: c.client_id,
	}
	data, err := json.Marshal(input)
	if err != nil {
		fmt.Println("Error marshaling capture input:", err)
		os.Exit(1)
	}
	exit, _, err := c.plugin.Call("flush", data)
	if err != nil {
		fmt.Println(err)
		os.Exit(int(exit))
	}
}
