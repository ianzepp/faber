// Package toml provides TOML encoding/decoding for the Faber HAL.
//
// Etymology: "TOML" - Tom's Obvious Minimal Language (universal, kept as-is).
package toml

import (
	"reflect"
	"strings"
	"time"

	"github.com/pelletier/go-toml/v2"
)

// =============================================================================
// SERIALIZATION
// =============================================================================

// Pange serializes table to TOML string.
func Pange(valor any) (string, error) {
	data, err := toml.Marshal(valor)
	if err != nil {
		return "", err
	}
	return string(data), nil
}

// =============================================================================
// PARSING
// =============================================================================

// Solve parses TOML string to table.
func Solve(tomlStr string) (any, error) {
	var result map[string]any
	err := toml.Unmarshal([]byte(tomlStr), &result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

// Tempta attempts to parse TOML string, returns nil on error.
func Tempta(tomlStr string) any {
	result, err := Solve(tomlStr)
	if err != nil {
		return nil
	}
	return result
}

// =============================================================================
// TYPE CHECKING
// =============================================================================

// EstNihil checks if value is nil.
func EstNihil(valor any) bool {
	return valor == nil
}

// EstBivalens checks if value is a boolean.
func EstBivalens(valor any) bool {
	_, ok := valor.(bool)
	return ok
}

// EstTextus checks if value is a string.
func EstTextus(valor any) bool {
	_, ok := valor.(string)
	return ok
}

// EstInteger checks if value is an integer.
func EstInteger(valor any) bool {
	switch valor.(type) {
	case int, int64, int32, int16, int8, uint, uint64, uint32, uint16, uint8:
		return true
	default:
		return false
	}
}

// EstFractus checks if value is a float.
func EstFractus(valor any) bool {
	switch valor.(type) {
	case float64, float32:
		return true
	default:
		return false
	}
}

// EstTempus checks if value is a time/datetime.
func EstTempus(valor any) bool {
	_, ok := valor.(time.Time)
	return ok
}

// EstLista checks if value is an array/slice.
func EstLista(valor any) bool {
	if valor == nil {
		return false
	}
	v := reflect.ValueOf(valor)
	return v.Kind() == reflect.Slice || v.Kind() == reflect.Array
}

// EstTabula checks if value is a table/map.
func EstTabula(valor any) bool {
	if valor == nil {
		return false
	}
	v := reflect.ValueOf(valor)
	return v.Kind() == reflect.Map
}

// =============================================================================
// VALUE EXTRACTION
// =============================================================================

// UtTextus extracts string value or returns default.
func UtTextus(valor any, defVal string) string {
	if s, ok := valor.(string); ok {
		return s
	}
	return defVal
}

// UtNumerus extracts numeric value or returns default.
func UtNumerus(valor any, defVal int) int {
	switch v := valor.(type) {
	case float64:
		return int(v)
	case float32:
		return int(v)
	case int:
		return v
	case int64:
		return int(v)
	case int32:
		return int(v)
	default:
		return defVal
	}
}

// UtBivalens extracts boolean value or returns default.
func UtBivalens(valor any, defVal bool) bool {
	if b, ok := valor.(bool); ok {
		return b
	}
	return defVal
}

// =============================================================================
// VALUE ACCESS
// =============================================================================

// Cape gets value by key from table (returns nil if missing).
func Cape(valor any, clavis string) any {
	if m, ok := valor.(map[string]any); ok {
		return m[clavis]
	}
	return nil
}

// Carpe plucks value by index from array (returns nil if out of bounds).
func Carpe(valor any, index int) any {
	if arr, ok := valor.([]any); ok {
		if index >= 0 && index < len(arr) {
			return arr[index]
		}
	}
	return nil
}

// Inveni finds value by dotted path (returns nil if not found).
func Inveni(valor any, via string) any {
	parts := strings.Split(via, ".")
	current := valor

	for _, part := range parts {
		if current == nil {
			return nil
		}
		if m, ok := current.(map[string]any); ok {
			current = m[part]
		} else {
			return nil
		}
	}

	return current
}
