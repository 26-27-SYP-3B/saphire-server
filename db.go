package main

import (
	"fmt"

	"github.com/glebarez/sqlite"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

// OpenDB opens the SQLite database at path and verifies the connection.
// Pass ":memory:" for a throwaway in-memory database (e.g. in tests).
func OpenDB(path string) (*gorm.DB, error) {
	db, err := gorm.Open(sqlite.Open(path), &gorm.Config{
		Logger: logger.Default.LogMode(logger.Silent),
	})
	if err != nil {
		return nil, fmt.Errorf("open sqlite %q: %w", path, err)
	}

	sqlDB, err := db.DB()
	if err != nil {
		return nil, fmt.Errorf("get *sql.DB: %w", err)
	}
	if path == ":memory:" {
		// An in-memory SQLite database lives inside a single connection.
		// Limit the pool to one so every query sees the same database.
		sqlDB.SetMaxOpenConns(1)
	}

	if err := sqlDB.Ping(); err != nil {
		return nil, fmt.Errorf("ping: %w", err)
	}

	return db, nil
}
