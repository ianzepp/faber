// Package yaml provides YAML encoding/decoding for the Faber HAL.
//
// Etymology: "YAML" - YAML Ain't Markup Language (universal, kept as-is).
package yaml

import (
	"reflect"
	"strings"

	"gopkg.in/yaml.v3"
)

// =============================================================================
// SERIALIZATION
// =============================================================================

// Pange serializes value to YAML string.
func Pange(valor any) (string, error) {
	data, err := yaml.Marshal(valor)
	if err != nil {
		return "", err
	}
	return string(data), nil
}

// Necto binds multiple documents into multi-doc YAML string.
func Necto(documenta []any) (string, error) {
	var parts []string
	for _, doc := range documenta {
		data, err := yaml.Marshal(doc)
		if err != nil {
			return "", err
		}
		parts = append(parts, string(data))
	}
	return strings.Join(parts, "---\n"), nil
}

// =============================================================================
// PARSING
// =============================================================================

// Solve parses YAML string to value.
func Solve(yamlStr string) (any, error) {
	var result any
	err := yaml.Unmarshal([]byte(yamlStr), &result)
	if err != nil {
		return nil, err
	}
	return normalizeYAML(result), nil
}

// Tempta attempts to parse YAML string, returns nil on error.
func Tempta(yamlStr string) any {
	result, err := Solve(yamlStr)
	if err != nil {
		return nil
	}
	return result
}

// Collige gathers all documents from multi-doc YAML string.
func Collige(yamlStr string) ([]any, error) {
	decoder := yaml.NewDecoder(strings.NewReader(yamlStr))
	var results []any

	for {
		var doc any
		err := decoder.Decode(&doc)
		if err != nil {
			if err.Error() == "EOF" {
				break
			}
			return nil, err
		}
		results = append(results, normalizeYAML(doc))
	}

	return results, nil
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

// EstNumerus checks if value is a number.
func EstNumerus(valor any) bool {
	switch valor.(type) {
	case float64, float32, int, int64, int32, int16, int8, uint, uint64, uint32, uint16, uint8:
		return true
	default:
		return false
	}
}

// EstTextus checks if value is a string.
func EstTextus(valor any) bool {
	_, ok := valor.(string)
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

// EstTabula checks if value is an object/map.
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

// Cape gets value by key from object (returns nil if missing).
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

// =============================================================================
// HELPERS
// =============================================================================

// normalizeYAML converts map[any]any to map[string]any for consistency.
// YAML v3 can produce map[string]any but we normalize to be safe.
func normalizeYAML(v any) any {
	switch val := v.(type) {
	case map[string]any:
		result := make(map[string]any)
		for k, v := range val {
			result[k] = normalizeYAML(v)
		}
		return result
	case map[any]any:
		result := make(map[string]any)
		for k, v := range val {
			if ks, ok := k.(string); ok {
				result[ks] = normalizeYAML(v)
			}
		}
		return result
	case []any:
		result := make([]any, len(val))
		for i, v := range val {
			result[i] = normalizeYAML(v)
		}
		return result
	default:
		return v
	}
}
