package main

import (
	"context"
	"fmt"
)

// App struct
type App struct {
	ctx context.Context
}

// NewApp creates a new App application struct
func NewApp() *App {
	return &App{}
}

// startup is called when the app starts. The context is saved
// so we can call the runtime methods
func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

type ApiResponse[T any] struct {
	Data  T      `json:"data"`
	Error string `json:"error"`
}

type GreetResponse struct {
	Message string `json:"message"`
}

func (a *App) Greet(name string) ApiResponse[GreetResponse] {
	return ApiResponse[GreetResponse]{
		Data:  GreetResponse{Message: fmt.Sprintf("Hello %s, It's show time!", name)},
		Error: "",
	}
}
