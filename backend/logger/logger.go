package logger

import (
	"archive/zip"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"
)

type Logger struct {
	mu           sync.Mutex
	dir          string
	file         *os.File
	sessionStart time.Time
	filename     string
}

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
	return l, nil
}

func (l *Logger) Close() error {
	l.mu.Lock()
	defer l.mu.Unlock()
	if l.file == nil {
		return nil
	}
	_ = l.file.Close()
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
	if l.filename == "" {
		stamp := l.sessionStart.Format("2006-01-02_15-04-05")
		l.filename = filepath.Join(l.dir, fmt.Sprintf("app-%s.jsonl", stamp))
	}
	f, err := os.OpenFile(l.filename, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
	if err != nil {
		return err
	}
	l.file = f
	return nil
}

func (l *Logger) appendJSON(level, msg string, fields map[string]any) {
	b, err := json.Marshal(logRecord{
		Time:    time.Now().UTC().Format(time.RFC3339Nano),
		Level:   level,
		Msg:     msg,
		Context: fields,
	})
	if err != nil || l.file == nil {
		return
	}
	if _, err := l.file.Write(append(b, '\n')); err != nil {
		return
	}
}

type logRecord struct {
	Time    string         `json:"time"`
	Level   string         `json:"level"`
	Msg     string         `json:"msg"`
	Context map[string]any `json:"context,omitempty"`
}

func (l *Logger) LogEndpoint(name string, request any, response any, errString string) {
	if err := l.ensure(); err != nil {
		return
	}
	l.mu.Lock()
	defer l.mu.Unlock()
	level := "INFO"
	if strings.TrimSpace(errString) != "" {
		level = "ERROR"
	}
	l.appendJSON(level, "backend.endpoint", map[string]any{
		"endpoint": name,
		"request":  request,
		"response": response,
		"error":    errString,
	})
}

func (l *Logger) LogFrontendEvent(name string, payload map[string]any) {
	if err := l.ensure(); err != nil {
		return
	}
	l.mu.Lock()
	defer l.mu.Unlock()
	l.appendJSON("INFO", "frontend.event", map[string]any{
		"name":    name,
		"payload": payload,
	})
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
