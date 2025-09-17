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

// Logger writes JSON logs to a per-session file and archives it on shutdown.
type Logger struct {
	mu           sync.Mutex
	dir          string
	file         *os.File
	sessionStart time.Time
	filename     string // full path to current session log file
	slogger      *slog.Logger
}

// NewLogger creates a new logger. Logs are stored under <dbDir>/logs.
func NewLogger(dbPath string) (*Logger, error) {
	dbDir := filepath.Dir(dbPath)
	logDir := filepath.Join(dbDir, "logs")
	if err := os.MkdirAll(logDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create logs dir: %w", err)
	}

	l := &Logger{dir: logDir, sessionStart: time.Now()}
	if err := l.openIfNeededLocked(); err != nil {
		return nil, err
	}
	l.slogger = slog.New(slog.NewJSONHandler(l.file, &slog.HandlerOptions{AddSource: true, Level: slog.LevelInfo}))
	return l, nil
}

// Close closes the underlying file.
func (l *Logger) Close() error {
	l.mu.Lock()
	defer l.mu.Unlock()
	if l.file == nil {
		return nil
	}
	// Close the file to flush all buffers
	_ = l.file.Close()
	// Archive current session log into a zip file and remove the original
	if l.filename != "" {
		archiveName := strings.TrimSuffix(l.filename, ".jsonl") + ".zip"
		if err := zipFiles(archiveName, []string{l.filename}); err != nil {
			return err
		}
		_ = os.Remove(l.filename)
	}
	l.file = nil
	return nil
}

func (l *Logger) ensure() error {
	l.mu.Lock()
	defer l.mu.Unlock()
	return l.openIfNeededLocked()
}

func (l *Logger) openIfNeededLocked() error {
	if l.file != nil {
		return nil
	}
	// Open session file once, named by program start date+time (YYYY-MM-DD_HH-MM-SS)
	if l.filename == "" {
		stamp := l.sessionStart.Format("2006-01-02_15-04-05")
		l.filename = filepath.Join(l.dir, fmt.Sprintf("app-%s.jsonl", stamp))
	}
	f, err := os.OpenFile(l.filename, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
	if err != nil {
		return err
	}
	l.file = f
	if l.slogger != nil {
		l.slogger = slog.New(slog.NewJSONHandler(l.file, &slog.HandlerOptions{AddSource: true, Level: slog.LevelInfo}))
	}
	return nil
}

// LogEndpoint writes a structured log for backend endpoint invocations.
func (l *Logger) LogEndpoint(name string, request any, response any, errString string) {
	_ = l.ensure()
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
	_ = l.ensure()
	l.slogger.LogAttrs(context.Background(), slog.LevelInfo, "frontend.event",
		slog.String("name", name),
		slog.Any("payload", payload),
	)
}

// Weekly archiving removed by design. Session logs are archived on shutdown.

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
