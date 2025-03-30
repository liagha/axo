# Axo Development Roadmap

## Current Focus (Q3 2024)
```mermaid
gantt
    title Axo Development Timeline
    dateFormat  YYYY-MM-DD
    section Core Language
    Lexer                     :active, des1, 2025-03-01, 60d
    Stream Parser (Token to Structured Flow AST) : active, des2, 2025-03-20, 30d
    Semantic Analysis         :active, des3, 2025-03-31, 30d
    
    Sema
    Type System Specification :         des3, after des1, 30d
    Memory Model              :         des3, after des2, 45d

    section Tooling
    Basic CLI Implementation  :         des4, 2024-08-01, 30d
    VS Code Extension         :         des5, after des4, 45d