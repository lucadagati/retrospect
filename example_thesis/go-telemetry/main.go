package main

import (
	"fmt"
	"io"
	"net/http"
	"strings"

	spinhttp "github.com/spinframework/spin-go-sdk/v2/http"
	"github.com/spinframework/spin-go-sdk/v2/kv"
)

const kvKey = "telemetry"

func init() {
	spinhttp.Handle(func(w http.ResponseWriter, r *http.Request) {
		store, err := kv.OpenStore("default")
		if err != nil {
			http.Error(w, fmt.Sprintf("kv open: %v", err), http.StatusInternalServerError)
			return
		}
		defer store.Close()

		switch r.Method {
		case http.MethodPost:
			body, _ := io.ReadAll(r.Body)
			newEntry := "[go] " + strings.TrimSpace(string(body))
			entries := loadEntries(store)
			entries = append(entries, newEntry)
			if err := store.Set(kvKey, []byte(strings.Join(entries, "\n"))); err != nil {
				http.Error(w, fmt.Sprintf("kv set: %v", err), http.StatusInternalServerError)
				return
			}
			w.WriteHeader(http.StatusOK)
			fmt.Fprintln(w, "go telemetry recorded")

		case http.MethodGet:
			entries := loadEntries(store)
			fmt.Fprintf(w, "=== Telemetry (go-telemetry view) ===\n")
			for i, e := range entries {
				fmt.Fprintf(w, "[%d] %s\n", i, e)
			}
			fmt.Fprintf(w, "total: %d entries\n", len(entries))

		default:
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	})
}

func main() {}

func loadEntries(store *kv.Store) []string {
	val, err := store.Get(kvKey)
	if err != nil || len(val) == 0 {
		return nil
	}
	var entries []string
	for _, line := range strings.Split(string(val), "\n") {
		if line != "" {
			entries = append(entries, line)
		}
	}
	return entries
}
