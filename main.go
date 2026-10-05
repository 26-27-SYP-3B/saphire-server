package main

import (
	"log"
	"net/http"
	"os"

	"github.com/gin-gonic/gin"
	"github.com/joho/godotenv"
	"gorm.io/gorm"
)

func newRouter(db *gorm.DB) *gin.Engine {
	r := gin.Default()

	r.GET("/ping", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{
			"message": "pong",
		})
	})

	r.GET("/health", func(c *gin.Context) {
		sqlDB, err := db.DB()
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"status": "error", "database": "unavailable"})
			return
		}
		if err := sqlDB.Ping(); err != nil {
			c.JSON(http.StatusServiceUnavailable, gin.H{"status": "degraded", "database": "unavailable"})
			return
		}
		c.JSON(http.StatusOK, gin.H{"status": "ok", "database": "ok"})
	})

	return r
}

func main() {
	// Load local configuration if a .env file exists; it is optional,
	// real deployments set these variables in the environment instead.
	_ = godotenv.Load()

	gin.SetMode(gin.ReleaseMode)

	dbPath := os.Getenv("SAPHIRE_DB_PATH")
	if dbPath == "" {
		dbPath = "saphire.db"
	}
	db, err := OpenDB(dbPath)
	if err != nil {
		log.Fatalf("database: %v", err)
	}
	log.Printf("database ready at %s", dbPath)

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	r := newRouter(db)
	log.Printf("listening on http://localhost:%s", port)
	if err := r.Run(":" + port); err != nil {
		log.Fatalf("http server: %v", err)
	}
}
