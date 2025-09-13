package logger

import (
	"archive/zip"
	"context"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"log/slog"
)

// Logger writes JSON logs to daily files and can archive old logs weekly.
type Logger struct {
	mu          sync.Mutex
	dir         string
	file        *os.File
	currentDate string // YYYY-MM-DD
	slogger     *slog.Logger
}

// NewLogger creates a new logger. Logs are stored under <dbDir>/logs.
func NewLogger(dbPath string) (*Logger, error) {
	dbDir := filepath.Dir(dbPath)
	logDir := filepath.Join(dbDir, "logs")
	if err := os.MkdirAll(logDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create logs dir: %w", err)
	}

	l := &Logger{dir: logDir}
	if err := l.rotateIfNeededLocked(time.Now()); err != nil {
		return nil, err
	}
	l.slogger = slog.New(slog.NewJSONHandler(l.file, &slog.HandlerOptions{AddSource: true, Level: slog.LevelInfo}))
	return l, nil
}

// Close closes the underlying file.
func (l *Logger) Close() error {
	l.mu.Lock()
	defer l.mu.Unlock()
	if l.file != nil {
		return l.file.Close()
	}
	return nil
}

func (l *Logger) ensure(today time.Time) error {
	l.mu.Lock()
	defer l.mu.Unlock()
	return l.rotateIfNeededLocked(today)
}

func (l *Logger) rotateIfNeededLocked(now time.Time) error {
	date := now.Format("2006-01-02")
	if l.file != nil && l.currentDate == date {
		return nil
	}
	// Close old file
	if l.file != nil {
		_ = l.file.Close()
		l.file = nil
	}

	filename := filepath.Join(l.dir, fmt.Sprintf("app-%s.jsonl", date))
	f, err := os.OpenFile(filename, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
	if err != nil {
		return err
	}
	l.file = f
	l.currentDate = date
	if l.slogger != nil {
		l.slogger = slog.New(slog.NewJSONHandler(l.file, &slog.HandlerOptions{AddSource: true, Level: slog.LevelInfo}))
	}
	return nil
}

// LogEndpoint writes a structured log for backend endpoint invocations.
func (l *Logger) LogEndpoint(name string, request any, response any, errString string) {
	_ = l.ensure(time.Now())
	level := slog.LevelInfo
	if strings.TrimSpace(errString) != "" {
		level = slog.LevelError
	}
	l.slogger.LogAttrs(context.Background(), level, "backend.endpoint",
		slog.String("endpoint", name),
		slog.Any("request", request),
		slog.Any("response", response),
		slog.String("error", errString),
	)
}

// LogFrontendEvent writes a structured log for frontend UI events.
func (l *Logger) LogFrontendEvent(name string, payload map[string]any) {
	_ = l.ensure(time.Now())
	l.slogger.LogAttrs(context.Background(), slog.LevelInfo, "frontend.event",
		slog.String("name", name),
		slog.Any("payload", payload),
	)
}

// ArchiveIfNeeded performs a weekly archive of all daily logs (except today's file).
// It uses a stamp file ".last_archive" to decide whether 7 days have passed since the last archive.
func (l *Logger) ArchiveIfNeeded() error {
	stampPath := filepath.Join(l.dir, ".last_archive")
	needArchive := false

	if data, err := os.ReadFile(stampPath); err == nil {
		if t, parseErr := time.Parse(time.RFC3339, strings.TrimSpace(string(data))); parseErr == nil {
			if time.Since(t) >= 7*24*time.Hour {
				needArchive = true
			}
		} else {
			// invalid stamp -> archive now
			needArchive = true
		}
	} else {
		// no stamp -> archive now
		needArchive = true
	}

	if !needArchive {
		return nil
	}

	entries, err := os.ReadDir(l.dir)
	if err != nil {
		return err
	}

	todayName := fmt.Sprintf("app-%s.jsonl", time.Now().Format("2006-01-02"))
	var filesToArchive []string
	for _, e := range entries {
		if e.IsDir() {
			continue
		}
		name := e.Name()
		if !strings.HasPrefix(name, "app-") || !strings.HasSuffix(name, ".jsonl") {
			continue
		}
		if name == todayName {
			// keep today's file open and unarchived
			continue
		}
		filesToArchive = append(filesToArchive, filepath.Join(l.dir, name))
	}

	if len(filesToArchive) == 0 {
		// still update stamp to avoid re-checking every startup
		_ = os.WriteFile(stampPath, []byte(time.Now().Format(time.RFC3339)), 0644)
		return nil
	}

	archiveName := filepath.Join(l.dir, fmt.Sprintf("logs-archive-%s.zip", time.Now().Format("20060102")))
	if err := zipFiles(archiveName, filesToArchive); err != nil {
		return err
	}
	// Remove originals
	for _, p := range filesToArchive {
		_ = os.Remove(p)
	}
	// Update stamp
	_ = os.WriteFile(stampPath, []byte(time.Now().Format(time.RFC3339)), 0644)
	return nil
}

func zipFiles(destination string, files []string) error {
	if len(files) == 0 {
		return errors.New("no files to zip")
	}
	out, err := os.Create(destination)
	if err != nil {
		return err
	}
	defer out.Close()

	zw := zip.NewWriter(out)
	defer zw.Close()

	for _, fpath := range files {
		if err := addFileToZip(zw, fpath); err != nil {
			return err
		}
	}
	return nil
}

func addFileToZip(zw *zip.Writer, path string) error {
	f, err := os.Open(path)
	if err != nil {
		return err
	}
	defer f.Close()

	info, err := f.Stat()
	if err != nil {
		return err
	}

	header, err := zip.FileInfoHeader(info)
	if err != nil {
		return err
	}
	header.Name = filepath.Base(path)
	header.Method = zip.Deflate

	w, err := zw.CreateHeader(header)
	if err != nil {
		return err
	}
	if _, err := io.Copy(w, f); err != nil {
		return err
	}
	return nil
}
