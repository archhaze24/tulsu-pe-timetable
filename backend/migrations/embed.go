package migrations

import "embed"

// FS содержит вшитые SQL-миграции
//
//go:embed *.sql
var FS embed.FS
