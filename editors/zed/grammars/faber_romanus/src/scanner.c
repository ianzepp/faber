#include "tree_sitter/parser.h"

enum TokenType {
  AUTOMATIC_SEMICOLON,
};

void *tree_sitter_faber_romanus_external_scanner_create() { return NULL; }
void tree_sitter_faber_romanus_external_scanner_destroy(void *p) {}
void tree_sitter_faber_romanus_external_scanner_reset(void *p) {}
unsigned tree_sitter_faber_romanus_external_scanner_serialize(void *p, char *buffer) { return 0; }
void tree_sitter_faber_romanus_external_scanner_deserialize(void *p, const char *b, unsigned n) {}

// Automatic semicolon insertion - allows statements without explicit semicolons
bool tree_sitter_faber_romanus_external_scanner_scan(void *payload, TSLexer *lexer,
                                                      const bool *valid_symbols) {
  if (valid_symbols[AUTOMATIC_SEMICOLON]) {
    // Skip whitespace except newlines
    while (lexer->lookahead == ' ' || lexer->lookahead == '\t' || lexer->lookahead == '\r') {
      lexer->advance(lexer, true);
    }

    // ASI on newline
    if (lexer->lookahead == '\n') {
      lexer->result_symbol = AUTOMATIC_SEMICOLON;
      return true;
    }

    // ASI at end of file
    if (lexer->eof(lexer)) {
      lexer->result_symbol = AUTOMATIC_SEMICOLON;
      return true;
    }

    // ASI before closing brace
    if (lexer->lookahead == '}') {
      lexer->result_symbol = AUTOMATIC_SEMICOLON;
      return true;
    }
  }

  return false;
}
