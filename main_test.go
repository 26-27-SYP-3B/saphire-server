package main

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
)

func setupRouter(t *testing.T) *gin.Engine {
	t.Helper()
	db, err := OpenDB(":memory:")
	if err != nil {
		t.Fatalf("open in-memory database: %v", err)
	}
	return newRouter(db)
}

func TestPing(t *testing.T) {
	r := setupRouter(t)

	req := httptest.NewRequest(http.MethodGet, "/ping", nil)
	w := httptest.NewRecorder()
	r.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Fatalf("expected status %d, got %d", http.StatusOK, w.Code)
	}

	if body := w.Body.String(); body != `{"message":"pong"}` {
		t.Fatalf("unexpected body: %s", body)
	}
}

func TestHealth(t *testing.T) {
	r := setupRouter(t)

	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	w := httptest.NewRecorder()
	r.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Fatalf("expected status %d, got %d", http.StatusOK, w.Code)
	}

	if body := w.Body.String(); body != `{"database":"ok","status":"ok"}` {
		t.Fatalf("unexpected body: %s", body)
	}
}
