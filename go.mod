module tulsu-pe-timetable

// Windows 7: собирайте с GOTOOLCHAIN=go1.20.14; на машине пользователя — WebView2 Runtime 109.x.
// После сборки фронта: GOOS=windows GOARCH=amd64 CGO_ENABLED=0 go build -trimpath .

go 1.20

require (
	github.com/pressly/goose/v3 v3.17.0
	github.com/wailsapp/wails/v2 v2.7.1
	github.com/xuri/excelize/v2 v2.8.1
	modernc.org/sqlite v1.29.10
)

require (
	github.com/bep/debounce v1.2.1 // indirect
	github.com/dustin/go-humanize v1.0.1 // indirect
	github.com/go-ole/go-ole v1.2.6 // indirect
	github.com/godbus/dbus/v5 v5.1.0 // indirect
	github.com/google/uuid v1.6.0 // indirect
	github.com/hashicorp/golang-lru/v2 v2.0.7 // indirect
	github.com/jchv/go-winloader v0.0.0-20210711035445-715c2860da7e // indirect
	github.com/labstack/echo/v4 v4.10.2 // indirect
	github.com/labstack/gommon v0.4.0 // indirect
	github.com/leaanthony/go-ansi-parser v1.6.0 // indirect
	github.com/leaanthony/gosod v1.0.3 // indirect
	github.com/leaanthony/slicer v1.6.0 // indirect
	github.com/leaanthony/u v1.1.0 // indirect
	github.com/mattn/go-colorable v0.1.13 // indirect
	github.com/mattn/go-isatty v0.0.20 // indirect
	github.com/mohae/deepcopy v0.0.0-20170929034955-c48cc78d4826 // indirect
	github.com/ncruces/go-strftime v0.1.9 // indirect
	github.com/pkg/browser v0.0.0-20210911075715-681adbf594b8 // indirect
	github.com/pkg/errors v0.9.1 // indirect
	github.com/remyoudompheng/bigfft v0.0.0-20230129092748-24d4a6f8daec // indirect
	github.com/richardlehane/mscfb v1.0.4 // indirect
	github.com/richardlehane/msoleps v1.0.3 // indirect
	github.com/rivo/uniseg v0.4.4 // indirect
	github.com/samber/lo v1.38.1 // indirect
	github.com/sethvargo/go-retry v0.2.4 // indirect
	github.com/tkrajina/go-reflector v0.5.6 // indirect
	github.com/valyala/bytebufferpool v1.0.0 // indirect
	github.com/valyala/fasttemplate v1.2.2 // indirect
	github.com/wailsapp/go-webview2 v1.0.10 // indirect
	github.com/wailsapp/mimetype v1.4.1 // indirect
	github.com/xuri/efp v0.0.0-20231025114914-d1ff6096ae53 // indirect
	github.com/xuri/nfp v0.0.0-20230919160717-d98342af3f05 // indirect
	go.uber.org/multierr v1.11.0 // indirect
	golang.org/x/crypto v0.19.0 // indirect
	golang.org/x/exp v0.0.0-20231108232855-2478ac86f678 // indirect
	golang.org/x/net v0.21.0 // indirect
	golang.org/x/sync v0.5.0 // indirect
	golang.org/x/sys v0.19.0 // indirect
	golang.org/x/text v0.14.0 // indirect
	modernc.org/gc/v3 v3.0.0-20240107210532-573471604cb6 // indirect
	modernc.org/libc v1.49.3 // indirect
	modernc.org/mathutil v1.6.0 // indirect
	modernc.org/memory v1.8.0 // indirect
	modernc.org/strutil v1.2.0 // indirect
	modernc.org/token v1.1.0 // indirect
)

// Keep the module graph usable with toolchain go1.20.14 (required for Windows 7 targets).
// Wails resolves newer minors of shared deps unless pinned here.
replace (
	github.com/labstack/echo/v4 => github.com/labstack/echo/v4 v4.10.2
	github.com/labstack/gommon => github.com/labstack/gommon v0.4.0
	github.com/pkg/browser => github.com/pkg/browser v0.0.0-20210911075715-681adbf594b8
	github.com/samber/lo => github.com/samber/lo v1.38.1
	github.com/wailsapp/go-webview2 => github.com/wailsapp/go-webview2 v1.0.10
	golang.org/x/crypto => golang.org/x/crypto v0.14.0
	golang.org/x/net => golang.org/x/net v0.17.0
	golang.org/x/sys => golang.org/x/sys v0.13.0
	golang.org/x/text => golang.org/x/text v0.13.0
)
