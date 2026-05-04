package main

import (
	"embed"
	"log"
	"tulsu-pe-timetable/backend/app_services"
	"tulsu-pe-timetable/backend/config"

	"github.com/wailsapp/wails/v2"
	"github.com/wailsapp/wails/v2/pkg/options"
	"github.com/wailsapp/wails/v2/pkg/options/assetserver"
	"github.com/wailsapp/wails/v2/pkg/options/windows"
)

//go:embed all:frontend/dist
var assets embed.FS

func main() {
	// Загружаем конфигурацию
	cfg, err := config.LoadConfig()
	if err != nil {
		log.Fatalf("Ошибка загрузки конфигурации: %v", err)
	}

	app, err := app_services.NewApp(cfg)
	if err != nil {
		log.Fatalf("Ошибка инициализации приложения: %v", err)
	}

	err = wails.Run(&options.App{
		Title:  "Расписание ТулГУ по физкультуре",
		Width:  1024,
		Height: 768,
		// Fixed WebView2 (например 109 под Win7): распакуйте runtime в каталог msedgewebview2.exe.
		Windows: &windows.Options{
			WebviewBrowserPath: `C:\WebView2`,
		},
		AssetServer: &assetserver.Options{
			Assets: assets,
		},
		BackgroundColour: &options.RGBA{R: 27, G: 38, B: 54, A: 1},
		OnStartup:        app.Startup,
		OnShutdown:       app.Shutdown,
		Bind: []interface{}{
			app,
		},
	})

	if err != nil {
		println("Error:", err.Error())
	}
}
